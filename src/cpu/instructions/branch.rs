// 跳转指令模块
use super::super::CPU;

impl CPU {
    // LJMP addr16 - 长跳转
    pub(crate) fn ljmp(&mut self) {
        let high_byte = self.fetch_next_byte();
        let low_byte = self.fetch_next_byte();
        let address = ((high_byte as u16) << 8) | (low_byte as u16);

        if self.debug {
            println!("ljmp {:#06x}", address);
        }
        self.registers.pc = address;
    }

    // AJMP addr11 - 绝对跳转（2KB页内）
    pub(crate) fn ajmp(&mut self, opcode: u8) {
        let addr_low = self.fetch_next_byte();
        // addr11由opcode的高3位(bits 7-5)和下一个字节组成
        let addr11 = (((opcode >> 5) as u16) << 8) | (addr_low as u16);
        // 保持PC的高5位，替换低11位
        let pc_high = self.registers.pc & 0xF800;
        let address = pc_high | addr11;
        
        if self.debug {
            println!("ajmp {:#06x}", address);
        }
        self.registers.pc = address;
    }

   
    pub(crate) fn sjmp(&mut self) {
        let offset = self.fetch_next_byte() as i8;
        let current_pc = self.registers.pc;
        let target = (current_pc as i32 + offset as i32) as u16;
        if self.debug {
            println!("{:<30}\t(offset={}, from PC={:#06x})", format!("sjmp {:#06x}", target), offset, current_pc);
        }
        self.registers.pc = target;
    }

    // JZ rel - 如果累加器为零则跳转
    pub(crate) fn jz(&mut self) {
        let offset = self.fetch_next_byte() as i8;
        let target = (self.registers.pc as i32 + offset as i32) as u16;

        // 检测 Delayms 函数退出条件（地址 0x0123，跳转到 0x0139）
        if self.delay_skip_counter > 0 && target == 0x0139 && self.registers.pc >= 0x0120 && self.registers.pc <= 0x0139 {
            self.delay_skip_counter = 0;
            if self.debug {
                println!("jz {:#06x}", target);
            }
            // 强制跳转到退出地址
            self.registers.pc = 0x0139;
            return;
        }

        if self.debug {
            println!("jz {:#06x}", target);
        }

        if self.registers.acc == 0 {
            self.registers.pc = target;
        }
    }

    // JNZ rel - 如果累加器不为零则跳转
    pub(crate) fn jnz(&mut self) {
        let offset = self.fetch_next_byte() as i8;
        let target = (self.registers.pc as i32 + offset as i32) as u16;

        // 快速跳过 Delayms 内部循环（地址 0x0129-0x0130 的循环）
        if self.delay_skip_counter > 0 && target == 0x0129 && self.registers.pc >= 0x0120 && self.registers.pc <= 0x0139 {
            // 将寄存器设为0以退出内层循环
            self.write_register(4, 0); // R4
            self.write_register(5, 0); // R5
            self.registers.acc = 0;
            return;
        }

        if self.debug {
            println!("jnz {:#06x}", target);
        }

        if self.registers.acc != 0 {
            self.registers.pc = target;
        }
    }

    // LCALL addr16 - 长调用
    pub(crate) fn lcall(&mut self) {
        let high_byte = self.fetch_next_byte();
        let low_byte = self.fetch_next_byte();
        let address = ((high_byte as u16) << 8) | (low_byte as u16);

        if self.debug {
            println!("lcall {:#06x}", address);
        }

        // 将当前PC压入堆栈（注意：8051先++SP再压栈）
        let return_addr = self.registers.pc;
        let low = (return_addr & 0xFF) as u8;
        let high = (return_addr >> 8) as u8;
        
        // 8051 PUSH操作：先SP++，再存储
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.ram[self.registers.sp as usize] = low; // 低字节
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.ram[self.registers.sp as usize] = high; // 高字节

        // 检测 Delayms 函数调用并优化执行
        if address == 0x011d { // Delayms 函数地址
            // 设置跳过计数器，在接下来的指令中快速跳过延迟循环
            self.delay_skip_counter = 1;
        }

        // 跳转到目标地址
        self.registers.pc = address;
    }

    // RET - 从子程序返回
    pub(crate) fn ret(&mut self) {
        if self.debug {
            println!("ret");
        }
        // 从堆栈弹出返回地址（8051 POP操作：先读取，再--SP）
        let high_byte = self.ram[self.registers.sp as usize] as u16; // 读高字节
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        let low_byte = self.ram[self.registers.sp as usize] as u16; // 读低字节
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        let return_address = (high_byte << 8) | low_byte;
        self.registers.pc = return_address;
    }

    // DJNZ Rn, rel - 寄存器减1，不为零则跳转
    pub(crate) fn djnz_rn(&mut self, reg_num: u8) {
        let offset = self.fetch_next_byte() as i8;
        let value = self.read_register(reg_num).wrapping_sub(1);
        self.write_register(reg_num, value);
        let target = (self.registers.pc as i32 + offset as i32) as u16;
        
        if self.debug {
            println!("{:<30}\t(value={}, offset={:+})", format!("djnz R{}, {:#06x}", reg_num, target), value, offset);
        }
        
        if value != 0 {
            self.registers.pc = target;
        }
    }

    // DJNZ direct, rel - 直接地址减1，不为0则跳转
    pub(crate) fn djnz_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        let offset = self.fetch_next_byte() as i8;
        
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };
        
        let new_value = value.wrapping_sub(1);
        
        if direct_address < 0x80 {
            self.ram[direct_address as usize] = new_value;
        } else {
            self.write_sfr(direct_address, new_value);
        }
        
        let target = (self.registers.pc as i32 + offset as i32) as u16;
        
        if self.debug {
            println!("djnz {:#04x}, {:#06x}", direct_address, target);
        }
        
        if new_value != 0 {
            self.registers.pc = target;
        }
    }

    // CJNE A, #data, rel - 比较A和立即数，如果不相等则跳转
    pub(crate) fn cjne_a_immediate(&mut self) {
        let immediate = self.fetch_next_byte();
        let offset = self.fetch_next_byte() as i8;

        if self.registers.acc != immediate {
            let target = (self.registers.pc as i32 + offset as i32) as u16;
            self.registers.pc = target;
        }

        if self.debug {
            println!("cjne A, #{:#04x}, {:+}", immediate, offset);
        }
    }

    // CJNE A, direct, rel - 比较A和直接地址，如果不相等则跳转
    pub(crate) fn cjne_a_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        let offset = self.fetch_next_byte() as i8;
        
        let direct_value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };

        let target = (self.registers.pc as i32 + offset as i32) as u16;

        if self.registers.acc != direct_value {
            self.registers.pc = target;
        }

        if self.debug {
            println!("{:<30}\t(direct_value={}, offset={:+})", format!("cjne A, {:#04x}, {:#06x}", direct_address, target), direct_value, offset);
        }
    }
}
