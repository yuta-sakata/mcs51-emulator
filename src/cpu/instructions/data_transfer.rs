// 数据传输指令模块
use super::super::CPU;
use super::{InstructionInfo, InstructionTable};

// 注册数据传输指令到指令表
pub fn register_instructions(table: &mut InstructionTable) {
    // MOV A, #data指令
    table[0x74] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_a_immediate(), mnemonic: "MOV" });
    
    // MOV A, direct指令
    table[0xE5] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_a_direct(), mnemonic: "MOV" });
    
    // MOV A, Rn指令 (0xE8-0xEF)
    for opcode in 0xE8..=0xEF {
        table[opcode] = Some(InstructionInfo { 
            handler: |cpu, op| cpu.mov_a_rn(op - 0xE8), 
            mnemonic: "MOV" 
        });
    }
    
    // MOV A, @Rn指令 (0xE6-0xE7)
    table[0xE6] = Some(InstructionInfo { 
        handler: |cpu, op| cpu.mov_a_rn_indirect(op - 0xE6), 
        mnemonic: "MOV" 
    });
    table[0xE7] = Some(InstructionInfo { 
        handler: |cpu, op| cpu.mov_a_rn_indirect(op - 0xE6), 
        mnemonic: "MOV" 
    });
    
    // MOV direct, A指令
    table[0xF5] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_direct_a(), mnemonic: "MOV" });
    
    // MOV direct, #data指令
    table[0x75] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_direct_immediate(), mnemonic: "MOV" });
    
    // MOV direct, direct指令
    table[0x85] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_direct_direct(), mnemonic: "MOV" });
    
    // MOV Rn, A指令 (0xF8-0xFF)
    for opcode in 0xF8..=0xFF {
        table[opcode] = Some(InstructionInfo { 
            handler: |cpu, op| cpu.mov_rn_a(op - 0xF8), 
            mnemonic: "MOV" 
        });
    }
    
    // MOV Rn, #data指令 (0x78-0x7F)
    for opcode in 0x78..=0x7F {
        table[opcode] = Some(InstructionInfo { 
            handler: |cpu, op| cpu.mov_rn_immediate(op - 0x78), 
            mnemonic: "MOV" 
        });
    }
    
    // MOV Rn, direct指令 (0xA8-0xAF)
    for opcode in 0xA8..=0xAF {
        table[opcode] = Some(InstructionInfo { 
            handler: |cpu, op| cpu.mov_rn_direct(op - 0xA8), 
            mnemonic: "MOV" 
        });
    }
    
    // MOV @Rn, A指令 (0xF6-0xF7)
    table[0xF6] = Some(InstructionInfo { 
        handler: |cpu, op| cpu.mov_rn_indirect_a(op - 0xF6), 
        mnemonic: "MOV" 
    });
    table[0xF7] = Some(InstructionInfo { 
        handler: |cpu, op| cpu.mov_rn_indirect_a(op - 0xF6), 
        mnemonic: "MOV" 
    });
    
    // MOV direct, Rn指令 (0x88-0x8F)
    for opcode in 0x88..=0x8F {
        table[opcode] = Some(InstructionInfo { 
            handler: |cpu, op| cpu.mov_direct_rn(op - 0x88), 
            mnemonic: "MOV" 
        });
    }
    
    // MOV DPTR, #data16指令
    table[0x90] = Some(InstructionInfo { handler: |cpu, _| cpu.mov_dptr_immediate(), mnemonic: "MOV" });
    
    // MOVX A, @DPTR指令
    table[0xE0] = Some(InstructionInfo { handler: |cpu, _| cpu.movx_a_dptr(), mnemonic: "MOVX" });
    
    // MOVX @DPTR, A指令
    table[0xF0] = Some(InstructionInfo { handler: |cpu, _| cpu.movx_dptr_a(), mnemonic: "MOVX" });
    
    // PUSH direct指令
    table[0xC0] = Some(InstructionInfo { handler: |cpu, _| cpu.push_direct(), mnemonic: "PUSH" });
    
    // POP direct指令
    table[0xD0] = Some(InstructionInfo { handler: |cpu, _| cpu.pop_direct(), mnemonic: "POP" });
    
    // CLR A指令
    table[0xE4] = Some(InstructionInfo { handler: |cpu, _| cpu.clr_acc(), mnemonic: "CLR" });
    
    // XCH A, direct指令
    table[0xC5] = Some(InstructionInfo { handler: |cpu, _| cpu.xch_a_direct(), mnemonic: "XCH" });
}

impl CPU {
    // PUSH direct - 将直接地址的内容压入堆栈
    pub(crate) fn push_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        
        // 读取直接地址的值
        let value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };
        
        // 8051 PUSH操作：先SP++，再存储
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.ram[self.registers.sp as usize] = value;
        
        if self.debug {
            println!("push {:#04x}", direct_address);
        }
    }

    // POP direct - 从堆栈弹出数据到直接地址
    pub(crate) fn pop_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        
        // 8051 POP操作：先读取，再--SP
        let value = self.ram[self.registers.sp as usize];
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        
        // 写入直接地址
        if direct_address < 0x80 {
            self.ram[direct_address as usize] = value;
        } else {
            self.write_sfr(direct_address, value);
        }
        
        if self.debug {
            println!("pop {:#04x}", direct_address);
        }
    }

    // CLR A - 清除累加器
    pub(crate) fn clr_acc(&mut self) {
        self.registers.acc = 0;
        if self.debug {
            println!("clr A");
        }
    }

    // MOV A, #data - 将立即数加载到累加器
    pub(crate) fn mov_a_immediate(&mut self) {
        let immediate = self.fetch_next_byte();
        self.registers.acc = immediate;
        if self.debug {
            println!("mov A, #{:#04x}", immediate);
        }
    }

    // MOV direct, #data - 将立即数存储到直接地址
    pub(crate) fn mov_direct_immediate(&mut self) {
        let direct_address = self.fetch_next_byte();
        let immediate = self.fetch_next_byte();

        if self.debug {
            println!("mov {:#04x}, #{:#04x}", direct_address, immediate);
        }

        if direct_address < 0x80 {
            self.ram[direct_address as usize] = immediate;
        } else {
            self.write_sfr(direct_address, immediate);
        }
    }

    // MOV A, direct - 将直接地址的值加载到累加器
    pub(crate) fn mov_a_direct(&mut self) {
        let direct_address = self.fetch_next_byte();

        if direct_address < 0x80 {
            self.registers.acc = self.ram[direct_address as usize];
        } else {
            self.registers.acc = self.read_sfr(direct_address);
        }

        if self.debug {
            println!("{:<30}\t(value={})", format!("mov A, {:#04x}", direct_address), self.registers.acc);
        }
    }

    // MOV direct, A - 将累加器存储到直接地址
    pub(crate) fn mov_direct_a(&mut self) {
        let direct_address = self.fetch_next_byte();

        if self.debug {
            println!("mov {:#04x}, A", direct_address);
        }

        if direct_address < 0x80 {
            self.ram[direct_address as usize] = self.registers.acc;
        } else {
            self.write_sfr(direct_address, self.registers.acc);
        }
    }

    // MOV direct, direct - 将一个直接地址的内容复制到另一个直接地址
    pub(crate) fn mov_direct_direct(&mut self) {
        let src_address = self.fetch_next_byte();
        let dst_address = self.fetch_next_byte();

        // 读取源地址的值
        let value = if src_address < 0x80 {
            self.ram[src_address as usize]
        } else {
            self.read_sfr(src_address)
        };

        if self.debug {
            println!("{:<30}\t(value={})", format!("mov {:#04x}, {:#04x}", dst_address, src_address), value);
        }

        // 写入目标地址
        if dst_address < 0x80 {
            self.ram[dst_address as usize] = value;
        } else {
            self.write_sfr(dst_address, value);
        }
    }

    // MOV Rn, #data - 将立即数加载到寄存器Rn
    pub(crate) fn mov_rn_immediate(&mut self, reg_num: u8) {
        let immediate = self.fetch_next_byte();
        self.write_register(reg_num, immediate);
        if self.debug {
            println!("mov R{}, #{:#04x}", reg_num, immediate);
        }
    }

    // MOV A, Rn - 将寄存器Rn加载到累加器
    pub(crate) fn mov_a_rn(&mut self, reg_num: u8) {
        self.registers.acc = self.read_register(reg_num);
        if self.debug {
            println!("{:<30}\t(value={})", format!("mov A, R{}", reg_num), self.registers.acc);
        }
    }

    // MOV Rn, A - 将累加器加载到寄存器Rn
    pub(crate) fn mov_rn_a(&mut self, reg_num: u8) {
        self.write_register(reg_num, self.registers.acc);
        if self.debug {
            println!("{:<30}\t(value={})", format!("mov R{}, A", reg_num), self.registers.acc);
        }
    }

    // MOV A, @Rn - 间接寻址，从Rn指向的地址读取到累加器
    pub(crate) fn mov_a_rn_indirect(&mut self, reg_num: u8) {
        let addr = self.read_register(reg_num) as usize;
        self.registers.acc = self.ram[addr];
        if self.debug {
            println!("mov A, @R{}", reg_num);
        }
    }

    // MOV @Rn, A - 间接寻址，将累加器写入Rn指向的地址
    pub(crate) fn mov_rn_indirect_a(&mut self, reg_num: u8) {
        let addr = self.read_register(reg_num) as usize;
        self.ram[addr] = self.registers.acc;
        if self.debug {
            println!("mov @R{}, A", reg_num);
        }
    }

    // MOV Rn, direct - 从直接地址加载到寄存器Rn
    pub(crate) fn mov_rn_direct(&mut self, reg_num: u8) {
        let direct = self.fetch_next_byte();
        let value = if direct < 0x80 {
            self.ram[direct as usize]
        } else {
            self.read_sfr(direct)
        };
        if self.debug {
            let reg_addr = self.get_register_address(reg_num);
            println!("{:<30}\t(value={}, will write to RAM[{}])", format!("mov R{}, {:#04x}", reg_num, direct), value, reg_addr);
        }
        self.write_register(reg_num, value);
    }

    // MOV DPTR, #data16 - 将16位立即数加载到数据指针
    pub(crate) fn mov_dptr_immediate(&mut self) {
        let high_byte = self.fetch_next_byte();
        let low_byte = self.fetch_next_byte();
        self.registers.dptr = ((high_byte as u16) << 8) | (low_byte as u16);
        if self.debug {
            println!("mov DPTR, #{:#06x}", self.registers.dptr);
        }
    }

    // MOV direct, Rn - 将寄存器Rn的值传送到直接地址
    pub(crate) fn mov_direct_rn(&mut self, reg_num: u8) {
        let direct_address = self.fetch_next_byte();
        let value = self.read_register(reg_num);
        
        if direct_address < 0x80 {
            self.ram[direct_address as usize] = value;
        } else {
            self.write_sfr(direct_address, value);
        }
        
        if self.debug {
            println!("mov {:#04x}, R{}", direct_address, reg_num);
        }
    }

    // MOVX @DPTR, A - 将累加器的值传送到DPTR指向的外部RAM
    pub(crate) fn movx_dptr_a(&mut self) {
        // 注意：这里简化处理，将外部RAM映射到内部ROM的高地址
        // 实际硬件中外部RAM是独立的
        let dptr = self.registers.dptr;
        if (dptr as usize) < self.rom.len() {
            self.rom[dptr as usize] = self.registers.acc;
        }
        
        if self.debug {
            println!("movx @DPTR, A");
        }
    }

    // MOVX A, @DPTR - 从DPTR指向的外部RAM读取到累加器
    pub(crate) fn movx_a_dptr(&mut self) {
        // 注意：这里简化处理，将外部RAM映射到内部ROM的高地址
        let dptr = self.registers.dptr;
        if (dptr as usize) < self.rom.len() {
            self.registers.acc = self.rom[dptr as usize];
        }
        
        if self.debug {
            println!("movx A, @DPTR");
        }
    }

    // XCH A, direct - 交换累加器和直接地址的内容
    pub(crate) fn xch_a_direct(&mut self) {
        let direct_address = self.fetch_next_byte();
        
        // 读取直接地址的值
        let direct_value = if direct_address < 0x80 {
            self.ram[direct_address as usize]
        } else {
            self.read_sfr(direct_address)
        };
        
        // 保存累加器的值
        let acc_value = self.registers.acc;
        
        // 交换值
        self.registers.acc = direct_value;
        
        if direct_address < 0x80 {
            self.ram[direct_address as usize] = acc_value;
        } else {
            self.write_sfr(direct_address, acc_value);
        }
        
        if self.debug {
            println!("xch A, {:#04x}", direct_address);
        }
    }
}
