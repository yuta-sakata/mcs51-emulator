pub mod arithmetic;
pub mod branch;
pub mod data_transfer;
pub mod interrupt;
pub mod logical;

use super::CPU;

impl CPU {
    pub fn execute_instruction(&mut self, opcode: u8, debug: bool, delay_skip_counter: &mut u32) {
        // 设置临时调试和优化标志
        self.debug = debug;
        self.delay_skip_counter = *delay_skip_counter;
        
        // 首先增加PC指向下一条指令
        self.registers.pc = self.registers.pc.wrapping_add(1);

        // 边界检查：确保PC在内存范围内
        if self.registers.pc as usize >= self.rom.len() {
            println!("错误: 程序计数器超出内存范围");
            return;
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
            0x23 => self.rl_a(), // RL A指令
            0x24 => self.add_acc_immediate(), // ADD A, #data指令
            0x25 => self.add_a_direct(), // ADD A, direct指令
            0x28..=0x2F => self.add_a_rn(opcode - 0x28), // ADD A, Rn指令
            0x30 => self.jnb_bit(), // JNB bit, rel指令
            0x32 => self.reti(), // RETI指令
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
            0xB2 => self.cpl_bit(), // CPL bit指令
            0xC0 => self.push_direct(), // PUSH direct指令
            0xC2 => self.clr_bit(), // CLR bit指令
            0xC3 => self.clr_c(), // CLR C指令
            0xC5 => self.xch_a_direct(), // XCH A, direct指令
            0xD0 => self.pop_direct(), // POP direct指令
            0xD2 => self.setb_bit(), // SETB bit指令
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
        
        // 将修改后的计数器写回
        *delay_skip_counter = self.delay_skip_counter;
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
        self.ram[addr] = value;
    }
}
