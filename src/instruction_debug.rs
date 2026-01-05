// 指令表调试和统计工具
// 独立于CPU实现，用于显示和分析指令表

use crate::cpu::instructions::{InstructionInfo, InstructionTable};
use crate::cpu::instructions::{arithmetic, branch, data_transfer, interrupt, logical};
use crate::cpu::CPU;

// 构建指令查找表
pub fn build_instruction_table() -> InstructionTable {
    let mut table: InstructionTable = [None; 256];
    
    // 委托给各个模块注册指令
    arithmetic::register_instructions(&mut table);
    branch::register_instructions(&mut table);
    data_transfer::register_instructions(&mut table);
    interrupt::register_instructions(&mut table);
    logical::register_instructions(&mut table);
    
    // NOP指令（通用指令，在这里注册）
    table[0x00] = Some(InstructionInfo {
        handler: |cpu, _| cpu.nop(),
        mnemonic: "NOP",
    });
    
    table
}

// 显示指令表（用于调试和统计）
pub fn dump_instruction_table() {
    let table = build_instruction_table();
    
    println!("[inst-dump] ===================================================================================================");
    println!("[inst-dump]       0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F");
    println!("[inst-dump] ===================================================================================================");
    
    for row in 0..16 {
        print!("[inst-dump]  {:X}0", row);
        
        for col in 0..16 {
            let opcode = (row * 16 + col) as usize;
            if let Some(info) = &table[opcode] {
                print!(" {:>5}", info.mnemonic);
            } else {
                print!("  ----");
            }
        }
        println!();
    }
    
    println!("[inst-dump] ===================================================================================================");
    
    let implemented = table.iter().filter(|x| x.is_some()).count();
    println!("[inst-dump] 已实现指令: {}/256 ({:.1}%)", 
             implemented, (implemented as f64 / 256.0) * 100.0);
}
