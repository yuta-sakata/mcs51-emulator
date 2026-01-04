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
    pub ram: [u8; 256],              // 内部RAM (0x00-0xFF)
    pub sfr: [u8; 128],              // 特殊功能寄存器 (0x80-0xFF)
    pub rom: [u8; 65536],            // 程序存储器 (64KB)
    pub interrupt_in_progress: bool, // 是否正在处理中断
    pub interrupt_return_pc: u16,    // 中断返回地址
    // 临时字段，用于在指令执行期间传递调试和优化信息
    pub(crate) debug: bool,          // 当前是否处于调试模式
    pub(crate) delay_skip_counter: u32, // 延迟跳过计数器（用于优化）
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            registers: Registers::new(),
            ram: [0; 256],
            sfr: [0; 128],
            rom: [0; 65536],
            interrupt_in_progress: false,
            interrupt_return_pc: 0,
            debug: false,
            delay_skip_counter: 0,
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


}
