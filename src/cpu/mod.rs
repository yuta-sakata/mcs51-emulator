pub mod instructions;
pub mod memory;
pub mod peripherals;
pub mod registers;

use hex::FromHexError;
use registers::Registers;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, Read};

#[derive(Debug)]
pub struct HexError(pub hex::FromHexError);

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hex decoding error: {}", self.0)
    }
}

impl std::error::Error for HexError {}

impl From<hex::FromHexError> for HexError {
    fn from(err: hex::FromHexError) -> Self {
        HexError(err)
    }
}

impl From<HexError> for std::io::Error {
    fn from(err: HexError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    }
}

pub struct CPU {
    pub registers: Registers,
    pub ram: [u8; 256], // 内部RAM (0x00-0xFF, 间接寻址可访问)
    pub register_banks: [[u8; 8]; 4],
    pub sfr: [u8; 128],              // 特殊功能寄存器 (0x80-0xFF, 直接寻址)
    pub rom: [u8; 65536],            // 程序存储器 (64KB)
    pub debug: bool,                 // 调试模式标志
    pub clock_frequency: u32,        // 时钟频率（Hz）
    pub clock_cycles: u64,           // 当前时钟周期计数
    pub delay_skip_counter: u32,     // 延迟跳过计数器
    pub loop_detector: LoopDetector, // 循环检测器
}

// 循环检测器：跟踪PC历史，识别紧密循环并快进
pub struct LoopDetector {
    pc_history: Vec<u16>,    // 最近的PC历史（用于检测循环）
    loop_count: u32,         // 当前循环已执行次数
    loop_start: u16,         // 循环起始地址
    loop_end: u16,           // 循环结束地址
    skip_threshold: u32,     // 触发快进的阈值（循环次数）
    fast_forward_count: u32, // 快进触发次数（用于检测重复快进）
}

impl LoopDetector {
    pub fn new() -> Self {
        LoopDetector {
            pc_history: Vec::with_capacity(100), // 存储最近100个PC
            loop_count: 0,                       // 当前循环计数
            loop_start: 0,                       // 循环起始地址
            loop_end: 0,                         // 循环结束地址
            skip_threshold: 10,                  // 循环超过10次就快进
            fast_forward_count: 0,               // 重复快进计数
        }
    }

    // 记录PC并检测循环模式
    pub fn record_pc(&mut self, pc: u16) -> bool {
        // 检测简单的后向跳转（循环的标志）
        if self.pc_history.len() > 0 {
            let last_pc = self.pc_history[self.pc_history.len() - 1];

            // 检测后向跳转（pc <= last_pc），增大检测范围以捕获外层循环
            if pc <= last_pc && last_pc.saturating_sub(pc) < 50 {
                // 如果是同一个循环
                if self.loop_start == pc || self.loop_start == 0 {
                    if self.loop_start == 0 {
                        self.loop_start = pc;
                        self.loop_end = last_pc;
                    }
                    self.loop_count += 1;

                    // 达到阈值时触发快进
                    if self.loop_count >= self.skip_threshold {
                        return true;
                    }
                } else {
                    // 检测到新循环，重置
                    self.loop_start = pc;
                    self.loop_end = last_pc;
                    self.loop_count = 1;
                }
            } else if pc > last_pc + 50 {
                // 跳出循环很远，重置
                if self.loop_count > 0 {
                    self.reset();
                }
            }
        }

        // 保持历史记录在合理大小
        if self.pc_history.len() > 50 {
            self.pc_history.remove(0);
        }
        self.pc_history.push(pc);
        false
    }

    pub fn reset(&mut self) {
        self.loop_count = 0;
    }

    pub fn get_fast_forward_multiplier(&self) -> u64 {
        // 如果同一个循环反复触发快进，说明是嵌套延时循环，加大快进力度
        if self.fast_forward_count > 10 {
            10_000_000_000 // 100亿个周期
        } else if self.fast_forward_count > 5 {
            1_000_000_000 // 10亿个周期
        } else if self.fast_forward_count > 2 {
            100_000_000 // 1亿个周期
        } else {
            10_000_000 // 1000万个周期
        }
    }

    pub fn increment_fast_forward(&mut self) {
        self.fast_forward_count += 1;
    }
}

impl CPU {
    pub fn new(debug: bool) -> Self {
        let mut cpu = CPU {
            registers: Registers::new(),        // 初始化寄存器
            ram: [0; 256],                      // 内部RAM (0x00-0xFF, 间接寻址可访问)
            register_banks: [[0; 8]; 4],        // 寄存器组
            sfr: [0; 128],                      // 特殊功能寄存器 (0x80-0xFF, 直接寻址)
            rom: [0; 65536],                    // 程序存储器 (64KB)
            debug,                              // 调试模式标志
            clock_frequency: 12000000,          // 默认12 MHz
            clock_cycles: 0,                    // 当前时钟周期计数
            delay_skip_counter: 0,              // 延迟跳过计数器
            loop_detector: LoopDetector::new(), // 循环检测器
        };
        // 初始化外设端口
        cpu.init_ports();
        cpu
    }

    // 从文件加载程序到内存
    pub fn load_program(&mut self, file_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // 将程序加载到内存
        for (i, &byte) in buffer.iter().enumerate() {
            if i < self.ram.len() {
                self.ram[i] = byte;
            } else {
                break; // 防止程序超出内存范围
            }
        }

        Ok(())
    }

    // 从Intel HEX文件加载程序到内存
    pub fn load_hex_program(&mut self, file_path: &str) -> io::Result<()> {
        let file = fs::File::open(file_path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if !line.starts_with(":") {
                continue; // 忽略无效行
            }

            // 解析HEX文件行
            let bytes = hex::decode(&line[1..]).map_err(HexError::from)?;
            let byte_count = bytes[0] as usize;
            let address = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
            let record_type = bytes[3];

            if record_type == 0x00 {
                // 数据记录
                for i in 0..byte_count {
                    let mem_address = address as usize + i;
                    if mem_address < self.rom.len() {
                        self.rom[mem_address] = bytes[4 + i];
                    }
                }
            } else if record_type == 0x01 {
                // 文件结束记录
                break;
            }
        }

        Ok(())
    }
}
