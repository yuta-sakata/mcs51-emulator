pub mod arithmetic;
pub mod branch;
pub mod data_transfer;
pub mod interrupt;
pub mod logical;

use super::CPU;

// 指令处理函数类型定义
pub type InstructionHandler = fn(&mut CPU, u8);

impl CPU {
    // 构建指令查找表
    fn build_instruction_table() -> [Option<InstructionHandler>; 256] {
        let mut table: [Option<InstructionHandler>; 256] = [None; 256];
        
        // 委托给各个模块注册指令
        arithmetic::register_instructions(&mut table);
        branch::register_instructions(&mut table);
        data_transfer::register_instructions(&mut table);
        interrupt::register_instructions(&mut table);
        logical::register_instructions(&mut table);
        
        // NOP指令（通用指令，在这里注册）
        table[0x00] = Some(|cpu, _| cpu.nop());
        
        table
    }
    
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
        static INSTRUCTION_TABLE: std::sync::OnceLock<[Option<InstructionHandler>; 256]> = std::sync::OnceLock::new();
        let table = INSTRUCTION_TABLE.get_or_init(|| Self::build_instruction_table());
        
        if let Some(handler) = table[opcode as usize] {
            handler(self, opcode);
        } else {
            println!("未知指令: 操作码 = {:#04x}", opcode);
        }
        
        // 将修改后的计数器写回
        *delay_skip_counter = self.delay_skip_counter;
    }
    
    // 获取已实现的指令数量（用于统计）
    pub fn get_implemented_instruction_count() -> usize {
        let table = Self::build_instruction_table();
        table.iter().filter(|x| x.is_some()).count()
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
