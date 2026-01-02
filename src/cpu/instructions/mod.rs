pub mod arithmetic;
pub mod branch;
pub mod data_transfer;
pub mod logical;

use super::CPU;

impl CPU {
    pub fn execute_instruction(&mut self, opcode: u8) {
        // 保存当前 PC 用于调试输出
        let pc_before = self.registers.pc;

        // 循环检测：如果检测到紧密循环超过阈值，快进
        if self.loop_detector.record_pc(pc_before) {
            self.loop_detector.increment_fast_forward();
            let multiplier = self.loop_detector.get_fast_forward_multiplier();

            if self.debug {
                println!(
                    "\n[LOOP FAST-FORWARD] 检测到紧密循环 ({:#06x}-{:#06x})，已执行 {} 次，快进 {} 个周期...",
                    self.loop_detector.loop_start,
                    self.loop_detector.loop_end,
                    self.loop_detector.loop_count,
                    multiplier
                );
            }

            // 快进：增加大量时钟周期
            self.clock_cycles += multiplier;

            // 非常小的循环（< 10字节）很可能是纯延时循环
            // 不要修改任何寄存器，只是让循环自然结束
            // 跳到循环结束之后继续
            self.registers.pc = self.loop_detector.loop_end.wrapping_add(1);

            self.loop_detector.reset();
            return;
        }

        // 首先增加PC指向下一条指令
        self.registers.pc = self.registers.pc.wrapping_add(1);

        // 边界检查：确保PC在内存范围内
        if self.registers.pc as usize >= self.rom.len() {
            println!("错误: 程序计数器超出内存范围");
            return;
        }

        // 每条指令增加机器周期（8051通常为12个时钟周期）
        self.clock_cycles += 12;

        // 在 debug 模式下，打印 [时钟周期][地址] 前缀
        if self.debug {
            print!("[{}][{:#06x}] ", self.clock_cycles, pc_before);
        }

        match opcode {
            0x00 => self.nop(), // NOP指令
            0x01 | 0x21 | 0x41 | 0x61 | 0x81 | 0xA1 | 0xC1 | 0xE1 => self.ajmp(opcode), // AJMP指令
            0x02 => self.ljmp(), // LJMP指令
            0x03 | 0x04 => self.inc_acc(), // INC A指令
            0x05 => self.inc_direct(), // INC direct指令
            0x08..=0x0F => self.inc_rn(opcode - 0x08), // INC Rn指令
            0x12 => self.lcall(), // LCALL指令
            0x13 => self.rrc_a(), // RRC A指令
            0x14 => self.dec_acc(), // DEC A指令
            0x18..=0x1F => self.dec_rn(opcode - 0x18), // DEC Rn指令
            0x22 => self.ret(), // RET指令
            0x24 => self.add_acc_immediate(), // ADD A, #data指令
            0x25 => self.add_a_direct(), // ADD A, direct指令
            0x28..=0x2F => self.add_a_rn(opcode - 0x28), // ADD A, Rn指令
            0x33 => self.rlc_a(), // RLC A指令
            0x34 => self.addc_acc_immediate(), // ADDC A, #data指令
            0x44 => self.orl_acc_immediate(), // ORL A, #data指令
            0x48..=0x4F => self.orl_a_rn(opcode - 0x48), // ORL A, Rn指令
            0x58..=0x5F => self.anl_a_rn(opcode - 0x58), // ANL A, Rn指令
            0x60 => self.jz(),  // JZ指令
            0x68..=0x6F => self.xrl_a_rn(opcode - 0x68), // XRL A, Rn指令
            0x70 => self.jnz(), // JNZ指令
            0x74 => self.mov_a_immediate(), // MOV A, #data指令
            0x75 => self.mov_direct_immediate(), // MOV direct, #data指令
            0x78..=0x7F => self.mov_rn_immediate(opcode - 0x78), // MOV Rn, #data指令
            0x80 => self.sjmp(), // SJMP指令
            0x82 => self.anl_direct_a(), // ANL direct, A指令
            0x84 => self.div_ab(), // DIV AB指令
            0x85 => self.mov_direct_direct(), // MOV direct, direct指令
            0x88..=0x8F => self.mov_direct_rn(opcode - 0x88), // MOV direct, Rn指令
            0x90 => self.mov_dptr_immediate(), // MOV DPTR, #data16指令
            0x95 => self.subb_a_direct(), // SUBB A, direct指令
            0x98..=0x9F => self.subb_a_rn(opcode - 0x98), // SUBB A, Rn指令
            0xA4 => self.mul_ab(), // MUL AB指令
            0xA8..=0xAF => self.mov_rn_direct(opcode - 0xA8), // MOV Rn, direct指令
            0xB5 => self.cjne_a_direct(), // CJNE A, direct, rel指令（注意0xB5和0xBE都是CJNE变体）
            0xBC => self.cjne_a_immediate(), // CJNE A, #data, rel指令
            0xBE => self.cjne_a_direct(), // CJNE A, direct, rel指令
            0xC3 => self.clr_c(), // CLR C指令
            0xD5 => self.djnz_direct(), // DJNZ direct, rel指令
            0xD8..=0xDF => self.djnz_rn(opcode - 0xD8), // DJNZ Rn, rel指令
            0xE0 => self.movx_a_dptr(), // MOVX A, @DPTR指令
            0xE4 => self.clr_acc(), // CLR A指令
            0xE5 => self.mov_a_direct(), // MOV A, direct指令
            0xE6 | 0xE7 => self.mov_a_rn_indirect(opcode - 0xE6), // MOV A, @Rn指令
            0xE8..=0xEF => self.mov_a_rn(opcode - 0xE8), // MOV A, Rn指令
            0xF0 => self.movx_dptr_a(), // MOVX @DPTR, A指令
            0xF4 => self.cpl_a(), // CPL A指令
            0xF5 => self.mov_direct_a(), // MOV direct, A指令
            0xF6 | 0xF7 => self.mov_rn_indirect_a(opcode - 0xF6), // MOV @Rn, A指令
            0xF8..=0xFF => self.mov_rn_a(opcode - 0xF8), // MOV Rn, A指令
            _ => println!("未知指令: 操作码 = {:#04x}", opcode),
        }
    }

    pub(crate) fn nop(&self) {
        if self.debug {
            println!("nop");
        }
    }

    pub(crate) fn get_carry_flag(&self) -> u8 {
        0 // 示例实现
    }

    // 辅助方法：获取当前寄存器组的寄存器地址
    pub(crate) fn get_register_address(&self, reg_num: u8) -> usize {
        // 当前寄存器组由PSW的RS1和RS0位决定，这里暂时使用组0
        let bank = 0; // 寄存器组0
        (bank * 8 + reg_num) as usize
    }

    // 读取寄存器Rn
    pub(crate) fn read_register(&self, reg_num: u8) -> u8 {
        let addr = self.get_register_address(reg_num);
        self.ram[addr]
    }

    // 写入寄存器Rn
    pub(crate) fn write_register(&mut self, reg_num: u8, value: u8) {
        let addr = self.get_register_address(reg_num);
        if self.debug && addr == 8 {
            println!(
                "WARNING: About to write to RAM[8] via register! reg_num={}, addr={}, value={}",
                reg_num, addr, value
            );
        }
        self.ram[addr] = value;
    }
}
