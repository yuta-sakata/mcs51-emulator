// 算术指令模块
use super::super::CPU;
use super::InstructionHandler;

// 注册算术指令到指令表
pub fn register_instructions(table: &mut [Option<InstructionHandler>; 256]) {
    // INC A指令 (0x03, 0x04)
    table[0x03] = Some(|cpu, _| cpu.inc_acc());
    table[0x04] = Some(|cpu, _| cpu.inc_acc());
    
    // INC direct指令
    table[0x05] = Some(|cpu, _| cpu.inc_direct());
    
    // INC Rn指令 (0x08-0x0F)
    for opcode in 0x08..=0x0F {
        table[opcode] = Some(|cpu, op| cpu.inc_rn(op - 0x08));
    }
    
    // DEC A指令
    table[0x14] = Some(|cpu, _| cpu.dec_acc());
    
    // DEC Rn指令 (0x18-0x1F)
    for opcode in 0x18..=0x1F {
        table[opcode] = Some(|cpu, op| cpu.dec_rn(op - 0x18));
    }
    
    // ADD A, #data指令
    table[0x24] = Some(|cpu, _| cpu.add_acc_immediate());
    
    // ADD A, direct指令
    table[0x25] = Some(|cpu, _| cpu.add_a_direct());
    
    // ADD A, Rn指令 (0x28-0x2F)
    for opcode in 0x28..=0x2F {
        table[opcode] = Some(|cpu, op| cpu.add_a_rn(op - 0x28));
    }
    
    // ADDC A, #data指令
    table[0x34] = Some(|cpu, _| cpu.addc_acc_immediate());
    
    // SUBB A, direct指令
    table[0x95] = Some(|cpu, _| cpu.subb_a_direct());
    
    // SUBB A, Rn指令 (0x98-0x9F)
    for opcode in 0x98..=0x9F {
        table[opcode] = Some(|cpu, op| cpu.subb_a_rn(op - 0x98));
    }
    
    // MUL AB指令
    table[0xA4] = Some(|cpu, _| cpu.mul_ab());
    
    // DIV AB指令
    table[0x84] = Some(|cpu, _| cpu.div_ab());
}

impl CPU {
    // INC A - 累加器加1
    pub(crate) fn inc_acc(&mut self) {
        self.registers.acc = self.registers.acc.wrapping_add(1);
        if self.debug {
            println!("inc A");
        }
    }

    // DEC A - 累加器减1
    pub(crate) fn dec_acc(&mut self) {
        self.registers.acc = self.registers.acc.wrapping_sub(1);
        if self.debug {
            println!("dec A");
        }
    }

    // DEC Rn - 寄存器Rn减1
    pub(crate) fn dec_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num).wrapping_sub(1);
        self.write_register(reg_num, value);
        if self.debug {
            println!("dec R{}", reg_num);
        }
    }

    // ADD A, #data - 累加器加立即数
    pub(crate) fn add_acc_immediate(&mut self) {
        let immediate = self.fetch_next_byte();
        self.registers.acc = self.registers.acc.wrapping_add(immediate);
        if self.debug {
            println!("add A, #{:#04x}", immediate);
        }
    }

    // ADD A, Rn - 累加器加寄存器Rn
    pub(crate) fn add_a_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num);
        let old_acc = self.registers.acc;
        self.registers.acc = self.registers.acc.wrapping_add(value);
        if self.debug {
            println!(
                "{:<30}\t(A: {} + R{}: {} = {})",
                format!("add A, R{}", reg_num), old_acc, reg_num, value, self.registers.acc
            );
        }
    }

    // ADD A, direct - 累加器加直接地址
    pub(crate) fn add_a_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };
        
        self.registers.acc = self.registers.acc.wrapping_add(value);
        
        if self.debug {
            println!("add A, {:#04x}", direct_address);
        }
    }

    // ADDC A, #data - 累加器加立即数加进位
    pub(crate) fn addc_acc_immediate(&mut self) {
        let immediate = self.fetch_next_byte();
        let carry = self.get_carry_flag();
        self.registers.acc = self
            .registers
            .acc
            .wrapping_add(immediate)
            .wrapping_add(carry);
        if self.debug {
            println!("addc A, #{:#04x}", immediate);
        }
    }

    // MUL AB - 乘法指令（使用加法模拟）
    pub(crate) fn mul_ab(&mut self) {
        let a = self.registers.acc;
        let b = self.registers.b;
        let mut result: u16 = 0;

        for _ in 0..b {
            result = result.wrapping_add(a as u16);
        }

        self.registers.acc = (result & 0xFF) as u8; // 低8位存入A
        self.registers.b = (result >> 8) as u8; // 高8位存入B

        if self.debug {
            println!("{:<30}\t(A = {}, B = {}, Result = {})", "mul AB", a, b, result);
        }
    }

    // DIV AB - 累加器除以B寄存器
    pub(crate) fn div_ab(&mut self) {
        let a = self.registers.acc;
        let b = self.read_sfr(0xF0); // B寄存器在0xF0

        if b == 0 {
            // 除以0，设置溢出标志
            let psw = self.read_sfr(0xD0);
            self.write_sfr(0xD0, psw | 0x04); // 设置OV位
        } else {
            let quotient = a / b;
            let remainder = a % b;

            self.registers.acc = quotient;
            self.write_sfr(0xF0, remainder); // 余数到B寄存器

            // 清除进位和溢出标志
            let psw = self.read_sfr(0xD0);
            self.write_sfr(0xD0, psw & 0x7B); // 清除CY和OV位
        }

        if self.debug {
            println!("div AB");
        }
    }

    // SUBB A, direct - 累加器减去直接地址和进位标志
    pub(crate) fn subb_a_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };

        let psw = self.read_sfr(0xD0);
        let carry = (psw >> 7) & 1; // 获取进位标志

        // 使用扩展精度计算以检测借位
        let acc = self.registers.acc as u16;
        let operand = (value as u16) + (carry as u16);
        let result = acc.wrapping_sub(operand);
        
        self.registers.acc = result as u8;
        
        // 设置进位标志：如果发生借位（acc < operand），CY = 1
        let new_psw = if acc < operand {
            psw | 0x80  // 设置CY位
        } else {
            psw & 0x7F  // 清除CY位
        };
        self.write_sfr(0xD0, new_psw);

        if self.debug {
            println!("subb A, {:#04x}", direct_address);
        }
    }

    // SUBB A, Rn - 累加器减去寄存器Rn和进位标志
    pub(crate) fn subb_a_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num);
        let psw = self.read_sfr(0xD0);
        let carry = (psw >> 7) & 1; // 获取进位标志
        
        // 使用扩展精度计算以检测借位
        let acc = self.registers.acc as u16;
        let operand = (value as u16) + (carry as u16);
        let result = acc.wrapping_sub(operand);
        
        self.registers.acc = result as u8;
        
        // 设置进位标志：如果发生借位（acc < operand），CY = 1
        let new_psw = if acc < operand {
            psw | 0x80  // 设置CY位
        } else {
            psw & 0x7F  // 清除CY位
        };
        self.write_sfr(0xD0, new_psw);
        
        if self.debug {
            println!("subb A, R{}", reg_num);
        }
    }

    // INC Rn - 寄存器加1
    pub(crate) fn inc_rn(&mut self, reg_num: u8) {
        let value = self.read_register(reg_num).wrapping_add(1);
        self.write_register(reg_num, value);

        if self.debug {
            println!("inc R{}", reg_num);
        }
    }

    // INC direct - 直接地址加1
    pub(crate) fn inc_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };
        
        let new_value = value.wrapping_add(1);
        
        if direct_address < 0x80 {
            self.ram[direct_address as usize] = new_value;
        } else {
            self.write_sfr(direct_address, new_value);
        }
        
        if self.debug {
            println!("inc {:#04x}", direct_address);
        }
    }
}
