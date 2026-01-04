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
    pub timer0_count: u16,           // 定时器0内部计数器
    pub timer1_count: u16,           // 定时器1内部计数器
    pub interrupt_in_progress: bool, // 是否正在处理中断
    pub interrupt_return_pc: u16,    // 中断返回地址
}

// 循环检测器：跟踪PC历史，识别紧密循环并智能快进
pub struct LoopDetector {
    pc_history: Vec<u16>,       // 最近的PC历史（用于检测循环）
    loop_count: u32,            // 当前循环已执行次数
    loop_start: u16,            // 循环起始地址
    loop_end: u16,              // 循环结束地址
    skip_threshold: u32,        // 触发快进的阈值（循环次数）
    fast_forward_count: u32,    // 快进触发次数（用于检测重复快进）
    has_io_in_loop: bool,       // 循环中是否有I/O操作
    io_operation_count: u32,    // 循环中I/O操作计数
    instructions_in_loop: u32,  // 循环中的指令数
    last_fast_forward_time: u64, // 上次快进时的时钟周期
}

impl LoopDetector {
    pub fn new() -> Self {
        LoopDetector {
            pc_history: Vec::with_capacity(100), // 存储最近100个PC
            loop_count: 0,                       // 当前循环计数
            loop_start: 0,                       // 循环起始地址
            loop_end: 0,                         // 循环结束地址
            skip_threshold: 100,                 // 循环超过100次才快进（观察期更长）
            fast_forward_count: 0,               // 重复快进计数
            has_io_in_loop: false,               // 默认无I/O
            io_operation_count: 0,               // I/O操作计数
            instructions_in_loop: 0,             // 循环指令数
            last_fast_forward_time: 0,           // 上次快进时间
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
            } else if pc > last_pc.saturating_add(50) {
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
        // 根据循环特征智能调整快进力度
        if self.has_io_in_loop {
            // 有I/O操作的循环：适度快进，保证输出频率真实
            // 快进到下一次I/O操作之前
            let cycles_per_loop = (self.instructions_in_loop as u64) * 12;
            let io_interval = if self.io_operation_count > 0 {
                cycles_per_loop * (self.loop_count as u64 / self.io_operation_count as u64).max(1)
            } else {
                cycles_per_loop * 10 // 快进10次循环
            };
            io_interval.min(100_000) // 最多快进10万周期
        } else {
            // 纯延时循环：大力快进，但保持时序准确
            // 根据嵌套深度加大快进力度
            if self.fast_forward_count > 10 {
                50_000_000 // 5000万周期 ≈ 4.17ms @ 12MHz
            } else if self.fast_forward_count > 5 {
                10_000_000 // 1000万周期 ≈ 0.83ms @ 12MHz
            } else if self.fast_forward_count > 2 {
                2_000_000  // 200万周期 ≈ 0.17ms @ 12MHz
            } else {
                500_000    // 50万周期 ≈ 0.04ms @ 12MHz
            }
        }
    }

    pub fn increment_fast_forward(&mut self) {
        self.fast_forward_count += 1;
    }

    // 记录循环中的I/O操作
    pub fn record_io_operation(&mut self) {
        self.io_operation_count += 1;
        self.has_io_in_loop = true;
    }

    // 记录循环中的指令数
    pub fn set_loop_size(&mut self, size: u32) {
        self.instructions_in_loop = size;
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
            timer0_count: 0,                    // 定时器0计数器
            timer1_count: 0,                    // 定时器1计数器
            interrupt_in_progress: false,       // 中断标志
            interrupt_return_pc: 0,             // 中断返回地址
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

    // 更新定时器（每个机器周期调用一次）
    pub fn update_timers(&mut self) {
        let tmod = self.sfr[0x09]; // TMOD寄存器 (0x89 - 0x80)
        let tcon = self.sfr[0x08]; // TCON寄存器 (0x88 - 0x80)

        // 定时器0更新
        let tr0 = (tcon & 0x10) != 0; // TR0位：定时器0运行控制
        if tr0 {
            let mode = tmod & 0x03; // 定时器0模式
            if mode == 0x01 {
                // 模式1：16位定时器/计数器
                let th0 = self.sfr[0x0C]; // TH0
                let tl0 = self.sfr[0x0A]; // TL0
                let mut count = ((th0 as u16) << 8) | (tl0 as u16);
                
                count = count.wrapping_add(1);
                
                // 检查溢出 (从0xFFFF回到0x0000)
                if count == 0 {
                    // 溢出，设置TF0标志
                    self.sfr[0x08] |= 0x20; // 设置TF0位(bit 5)
                }
                
                // 更新TH0和TL0的值（反映当前计数）
                self.sfr[0x0C] = (count >> 8) as u8; // TH0
                self.sfr[0x0A] = (count & 0xFF) as u8; // TL0
            }
        }
    }

    // 检查并处理中断
    pub fn check_interrupts(&mut self) -> bool {
        let ie = self.sfr[0x28];  // IE寄存器 (0xA8 - 0x80)
        let ea = (ie & 0x80) != 0;     // EA位：总中断使能
        
        if !ea {
            return false; // 总中断未使能
        }
        
        if self.interrupt_in_progress {
            return false; // 正在处理中断
        }

        let tcon = self.sfr[0x08]; // TCON寄存器 (0x88 - 0x80)
        let et0 = (ie & 0x02) != 0;     // ET0位：定时器0中断使能
        let tf0 = (tcon & 0x20) != 0;   // TF0位：定时器0溢出标志

        // 检查定时器0中断
        if et0 && tf0 {
            // 清除TF0标志
            self.sfr[0x08] &= !0x20; // 清除TF0
            
            // 保存当前PC到堆栈（先压低字节，再压高字节）
            self.push_stack((self.registers.pc & 0xFF) as u8);
            self.push_stack((self.registers.pc >> 8) as u8);
            
            // 跳转到中断向量（定时器0在0x000B）
            self.interrupt_return_pc = self.registers.pc;
            self.registers.pc = 0x000B;
            self.interrupt_in_progress = true;
            
            return true;
        }

        false
    }

    // 辅助函数：压栈
    fn push_stack(&mut self, value: u8) {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.ram[self.registers.sp as usize] = value;
    }

    // 辅助函数：出栈
    fn pop_stack(&mut self) -> u8 {
        let value = self.ram[self.registers.sp as usize];
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        value
    }
}
