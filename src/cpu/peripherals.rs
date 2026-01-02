// 8051 外设模块
// 实现 I/O 端口 (P0-P3) 和其他外设功能

use super::CPU;

// SFR 地址定义
pub const P0: u8 = 0x80;  // 端口 0
pub const P1: u8 = 0x90;  // 端口 1
pub const P2: u8 = 0xA0;  // 端口 2
pub const P3: u8 = 0xB0;  // 端口 3
pub const PSW: u8 = 0xD0; // 程序状态字
pub const ACC: u8 = 0xE0; // 累加器
pub const B: u8 = 0xF0;   // 寄存器 B

impl CPU {
    /// 读取 SFR 寄存器（带外设处理）
    pub fn read_sfr(&self, address: u8) -> u8 {
        match address {
            P0 => {
                // println!("读取P0端口: {:#04x}", self.sfr[(P0 - 0x80) as usize]);
                self.sfr[(P0 - 0x80) as usize]
            }
            P1 => {
                // println!("读取P1端口: {:#04x}", self.sfr[(P1 - 0x80) as usize]);
                self.sfr[(P1 - 0x80) as usize]
            }
            P2 => {
                // println!("读取P2端口: {:#04x}", self.sfr[(P2 - 0x80) as usize]);
                self.sfr[(P2 - 0x80) as usize]
            }
            P3 => {
                // println!("读取P3端口: {:#04x}", self.sfr[(P3 - 0x80) as usize]);
                self.sfr[(P3 - 0x80) as usize]
            }
            ACC => self.registers.acc, // 累加器映射到 SFR
            B => self.registers.b,     // B 寄存器映射到 SFR
            0x81 => self.registers.sp, // SP (Stack Pointer)
            _ => {
                if address >= 0x80 {
                    self.sfr[(address - 0x80) as usize]
                } else {
                    0
                }
            }
        }
    }

    /// 写入 SFR 寄存器（带外设处理）
    pub fn write_sfr(&mut self, address: u8, value: u8) {
        match address {
            P0 => {
                if !self.debug {
                    println!("写入P0端口: {:#04x} (二进制: {:08b})", value, value);
                }
                self.sfr[(P0 - 0x80) as usize] = value;
                self.handle_port_output(0, value);
            }
            P1 => {
                if !self.debug {
                    println!("写入P1端口: {:#04x} (二进制: {:08b})", value, value);
                }
                self.sfr[(P1 - 0x80) as usize] = value;
                self.handle_port_output(1, value);
            }
            P2 => {
                if !self.debug {
                    println!("写入P2端口: {:#04x} (二进制: {:08b})", value, value);
                }
                self.sfr[(P2 - 0x80) as usize] = value;
                self.handle_port_output(2, value);
            }
            P3 => {
                if !self.debug {
                    println!("写入P3端口: {:#04x} (二进制: {:08b})", value, value);
                }
                self.sfr[(P3 - 0x80) as usize] = value;
                self.handle_port_output(3, value);
            }
            ACC => {
                self.registers.acc = value; // 累加器映射到 SFR
                self.sfr[(ACC - 0x80) as usize] = value;
            }
            B => {
                self.registers.b = value;   // B 寄存器映射到 SFR
                self.sfr[(B - 0x80) as usize] = value;
            }
            0x81 => {
                // SP (Stack Pointer)
                self.registers.sp = value;
                self.sfr[(0x81 - 0x80) as usize] = value;
            }
            _ => {
                if address >= 0x80 {
                    self.sfr[(address - 0x80) as usize] = value;
                }
            }
        }
    }

    /// 处理端口输出（模拟外设行为）
    fn handle_port_output(&self, port_num: u8, value: u8) {
        // 这里可以添加更多的外设模拟逻辑
        // 例如：LED显示、LCD控制、继电器开关等
        
        // 示例：检测特定位的变化
        if port_num == 1 {
            // P1 端口可能连接 LED
            for bit in 0..8 {
                let bit_value = (value >> bit) & 1;
                if bit_value == 1 {
                    // println!("  P1.{} = 高电平 (LED点亮)", bit);
                } else {
                    // println!("  P1.{} = 低电平 (LED熄灭)", bit);
                }
            }
        }
    }

    /// 初始化所有端口为默认值
    pub fn init_ports(&mut self) {
        // 8051 复位后，所有端口默认为 0xFF (全高)
        self.sfr[(P0 - 0x80) as usize] = 0xFF;
        self.sfr[(P1 - 0x80) as usize] = 0xFF;
        self.sfr[(P2 - 0x80) as usize] = 0xFF;
        self.sfr[(P3 - 0x80) as usize] = 0xFF;
    }
}
