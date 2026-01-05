pub mod arithmetic;
pub mod branch;
pub mod data_transfer;
pub mod interrupt;
pub mod logical;

use super::CPU;

// 指令信息结构
#[derive(Clone, Copy)]
pub struct InstructionInfo {
    pub handler: fn(&mut CPU, u8),
    pub mnemonic: &'static str,
}

// 指令表类型定义
pub type InstructionTable = [Option<InstructionInfo>; 256];

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

        // 使用静态查找表执行指令
        static INSTRUCTION_TABLE: std::sync::OnceLock<InstructionTable> = std::sync::OnceLock::new();
        let table = INSTRUCTION_TABLE.get_or_init(|| crate::instruction_debug::build_instruction_table());
        
        if let Some(info) = &table[opcode as usize] {
            (info.handler)(self, opcode);
        } else {
            println!("未知指令: 操作码 = {:#04x}", opcode);
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
