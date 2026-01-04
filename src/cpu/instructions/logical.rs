// 逻辑指令模块
use super::super::CPU;

impl CPU {
    // ORL A, #data - 累加器与立即数进行逻辑或
    pub(crate) fn orl_acc_immediate(&mut self) {
        let immediate = self.fetch_next_byte();
        self.registers.acc |= immediate;
        if self.debug {
            println!("orl A, #{:#04x}", immediate);
        }
    }

    // ORL A, Rn - 累加器与寄存器Rn进行逻辑或
    pub(crate) fn orl_a_rn(&mut self, reg_num: u8) {
        self.registers.acc |= self.read_register(reg_num);
        if self.debug {
            println!("orl A, R{}", reg_num);
        }
    }

    // ANL direct, A - 直接地址与累加器进行逻辑与
    pub(crate) fn anl_direct_a(&mut self) {
        let direct_address = self.fetch_next_byte();
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };

        self.registers.acc &= value;

        if self.debug {
            println!("anl {:#04x}, A", direct_address);
        }
    }

    // CLR C - 清除进位标志
    pub(crate) fn clr_c(&mut self) {
        // PSW寄存器在地址0xD0，进位标志是bit 7
        let psw = self.read_sfr(0xD0);
        self.write_sfr(0xD0, psw & 0x7F); // 清除bit 7 (CY位)
        
        if self.debug {
            println!("clr C");
        }
    }

    // ANL A, Rn - 累加器与寄存器Rn进行逻辑与
    pub(crate) fn anl_a_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num);
        self.registers.acc &= value;
        if self.debug {
            println!("anl A, R{}", reg_num);
        }
    }

    // XRL A, Rn - 累加器与寄存器Rn进行逻辑异或
    pub(crate) fn xrl_a_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num);
        self.registers.acc ^= value;
        if self.debug {
            println!("xrl A, R{}", reg_num);
        }
    }

    // CPL A - 累加器按位取反
    pub(crate) fn cpl_a(&mut self) {
        self.registers.acc = !self.registers.acc;
        if self.debug {
            println!("cpl A");
        }
    }

    // RLC A - 累加器左移循环通过进位
    pub(crate) fn rlc_a(&mut self) {
        let psw = self.read_sfr(0xD0);
        let old_carry = (psw >> 7) & 1;
        let new_carry = (self.registers.acc >> 7) & 1;
        
        self.registers.acc = (self.registers.acc << 1) | old_carry;
        
        // 更新进位标志
        let new_psw = if new_carry == 1 {
            psw | 0x80
        } else {
            psw & 0x7F
        };
        self.write_sfr(0xD0, new_psw);
        
        if self.debug {
            println!("rlc A");
        }
    }

    // RL A - 累加器左移（不通过进位）
    pub(crate) fn rl_a(&mut self) {
        let carry_out = (self.registers.acc >> 7) & 1;
        self.registers.acc = (self.registers.acc << 1) | carry_out;
        
        if self.debug {
            println!("rl A");
        }
    }

    // RRC A - 累加器右移循环通过进位
    pub(crate) fn rrc_a(&mut self) {
        let psw = self.read_sfr(0xD0);
        let old_carry = (psw >> 7) & 1;
        let new_carry = self.registers.acc & 1;
        
        self.registers.acc = (self.registers.acc >> 1) | (old_carry << 7);
        
        // 更新进位标志
        let new_psw = if new_carry == 1 {
            psw | 0x80
        } else {
            psw & 0x7F
        };
        self.write_sfr(0xD0, new_psw);
        
        if self.debug {
            println!("rrc A");
        }
    }

    // SETB bit - 设置指定的位
    pub(crate) fn setb_bit(&mut self) {
        let bit_addr = self.fetch_next_byte();
        
        // 位地址 0x00-0x7F 对应 RAM 的 0x20-0x2F (位寻址区)
        // 位地址 0x80-0xFF 对应 SFR 的位寻址区
        if bit_addr < 0x80 {
            // 内部RAM位寻址
            let byte_addr = 0x20 + (bit_addr >> 3) as usize;
            let bit_pos = bit_addr & 0x07;
            self.ram[byte_addr] |= 1 << bit_pos;
        } else {
            // SFR位寻址
            // SFR位地址映射：0x80-0x87对应0x80, 0x88-0x8F对应0x88, 0x90-0x97对应0x90, ...
            let byte_addr = (bit_addr & 0xF8);  // 取高5位得到字节地址
            let bit_pos = bit_addr & 0x07;
            let value = self.read_sfr(byte_addr);
            self.write_sfr(byte_addr, value | (1 << bit_pos));
        }
        
        if self.debug {
            println!("setb {:#04x}", bit_addr);
        }
    }

    // CPL bit - 对指定的位取反
    pub(crate) fn cpl_bit(&mut self) {
        let bit_addr = self.fetch_next_byte();
        
        // 位地址 0x00-0x7F 对应 RAM 的 0x20-0x2F (位寻址区)
        // 位地址 0x80-0xFF 对应 SFR 的位寻址区
        if bit_addr < 0x80 {
            // 内部RAM位寻址
            let byte_addr = 0x20 + (bit_addr >> 3) as usize;
            let bit_pos = bit_addr & 0x07;
            self.ram[byte_addr] ^= 1 << bit_pos; // 异或实现取反
        } else {
            // SFR位寻址
            let byte_addr = (bit_addr & 0xF8);  // 取高5位得到字节地址
            let bit_pos = bit_addr & 0x07;
            let value = self.read_sfr(byte_addr);
            self.write_sfr(byte_addr, value ^ (1 << bit_pos)); // 异或实现取反
        }
        
        if self.debug {
            println!("cpl {:#04x}", bit_addr);
        }
    }

    // CLR bit - 清除指定的位
    pub(crate) fn clr_bit(&mut self) {
        let bit_addr = self.fetch_next_byte();
        
        // 位地址 0x00-0x7F 对应 RAM 的 0x20-0x2F (位寻址区)
        // 位地址 0x80-0xFF 对应 SFR 的位寻址区
        if bit_addr < 0x80 {
            // 内部RAM位寻址
            let byte_addr = 0x20 + (bit_addr >> 3) as usize;
            let bit_pos = bit_addr & 0x07;
            self.ram[byte_addr] &= !(1 << bit_pos);
        } else {
            // SFR位寻址
            let byte_addr = (bit_addr & 0xF8);  // 取高5位得到字节地址
            let bit_pos = bit_addr & 0x07;
            let value = self.read_sfr(byte_addr);
            self.write_sfr(byte_addr, value & !(1 << bit_pos));
        }
        
        if self.debug {
            println!("clr {:#04x}", bit_addr);
        }
    }
}
