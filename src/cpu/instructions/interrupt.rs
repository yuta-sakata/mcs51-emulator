// 中断处理模块
use super::super::CPU;
use super::{InstructionInfo, InstructionTable};

// 注册中断指令到指令表
pub fn register_instructions(table: &mut InstructionTable) {
    // RETI指令
    table[0x32] = Some(InstructionInfo { handler: |cpu, _| cpu.reti(), mnemonic: "RETI" });
}

impl CPU {
    // RETI - 从中断返回
    pub(crate) fn reti(&mut self) {
        // 从堆栈弹出返回地址
        let high_byte = self.ram[self.registers.sp as usize] as u16;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        let low_byte = self.ram[self.registers.sp as usize] as u16;
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        let return_address = (high_byte << 8) | low_byte;

        if self.debug {
            println!("reti");
        }

        self.registers.pc = return_address;

        // 清除中断标志
        self.interrupt_in_progress = false;
    }

    // 检查并处理中断
    pub fn check_interrupts(&mut self) -> bool {
        let ie = self.sfr[0x28]; // IE寄存器 (0xA8 - 0x80)
        let ea = (ie & 0x80) != 0; // EA位：总中断使能

        if !ea {
            return false; // 总中断未使能
        }

        if self.interrupt_in_progress {
            return false; // 正在处理中断
        }

        let tcon = self.sfr[0x08]; // TCON寄存器 (0x88 - 0x80)
        let et0 = (ie & 0x02) != 0; // ET0位：定时器0中断使能
        let tf0 = (tcon & 0x20) != 0; // TF0位：定时器0溢出标志

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
    pub(crate) fn push_stack(&mut self, value: u8) {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.ram[self.registers.sp as usize] = value;
    }

    // 辅助函数：出栈
    pub(crate) fn pop_stack(&mut self) -> u8 {
        let value = self.ram[self.registers.sp as usize];
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        value
    }
}
