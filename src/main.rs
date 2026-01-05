mod cpu;
mod emulator;
mod loop_detector;
mod instruction_debug;

use emulator::Emulator;
use std::env;
use std::path::Path;
use std::process;

/**
 *                             _ooOoo_
 *                            o8888888o
 *                            88" . "88
 *                            (| -_- |)
 *                            O\  =  /O
 *                         ____/`---'\____
 *                       .'  \\|     |//  `.
 *                      /  \\|||  :  |||//  \
 *                     /  _||||| -:- |||||-  \
 *                     |   | \\\  -  /// |   |
 *                     | \_|  ''\---/''  |   |
 *                     \  .-\__  `-`  ___/-. /
 *                   ___`. .'  /--.--\  `. . __
 *                ."" '<  `.___\_<|>_/___.'  >'"".
 *               | | :  `- \`.;`\ _ /`;.`/ - ` : | |
 *               \  \ `-.   \_ __\ /__ _/   .-` /  /
 *          ======`-.____`-.___\_____/___.-`____.-'======
 *                             `=---='
 *          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 *                     佛祖保佑        永无BUG
 */

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    // 检查是否是帮助模式
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help(&args[0]);
        return;
    }
    
    // 检查是否是指令表转储模式
    if args.iter().any(|arg| arg == "--inst-dump" || arg == "-i") {
        instruction_debug::dump_instruction_table();
        return;
    }
    
    if args.len() < 2 {
        eprintln!("用法: {} <程序文件> [选项]", args[0]);
        eprintln!("使用 --help 或 -h 查看详细帮助信息");
        process::exit(1);
    }

    let hex_file = &args[1];
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "debug");

    // 初始化模拟器
    let mut emulator = Emulator::new(debug_mode);

    //HEX文件加载程序
    match emulator.cpu.load_hex_program(hex_file) {
        Ok(_) => println!("程序成功从 {} 加载", hex_file),
        Err(e) => {
            eprintln!("加载程序失败: {}", e);
            process::exit(1);
        }
    }
    
    loop {
        // 检查是否已停机
        if emulator.is_halted {
            if !debug_mode {
                println!("\n程序执行完成");
            }
            break;
        }

        // 检查PC是否在有效范围内
        if (emulator.cpu.registers.pc as usize) >= emulator.cpu.rom.len() {
            println!("程序结束: PC超出内存范围");
            break;
        }
        
        // 检查指令执行数限制（防止真正的无限循环）
        if emulator.instruction_count > 100_000_000 {
            println!("\n警告: 已执行超过1亿条指令，可能存在死循环，强制退出");
            break;
        }
        
        let pc = emulator.cpu.registers.pc;
        let opcode = emulator.cpu.rom[pc as usize];
        emulator.execute_instruction(opcode);
        
        // 更新定时器（每条指令执行后）
        emulator.cpu.update_timers();
        
        // 检查并处理中断
        emulator.cpu.check_interrupts();
    }

    // 打印最终状态
    println!("CPU 状态：累加器 = {}, 程序计数器 = {}", emulator.cpu.registers.acc, emulator.cpu.registers.pc);
}

fn print_help(program_name: &str) {
    let prog_name = Path::new(program_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mcs51-emulator");
    
    println!("MCS-51 单片机模拟器");
    println!();
    println!("用法:");
    println!("  {} <程序文件> [选项]          运行 Intel HEX 格式的程序", prog_name);
    println!("  {} --inst-dump                显示指令实现情况统计表", prog_name);
    println!("  {} --help                     显示此帮助信息", prog_name);
    println!();
    println!("选项:");
    println!("  --debug, debug                启用调试模式，显示每条指令执行信息");
    println!("  --inst-dump, -i               显示已实现的指令统计表");
    println!("  --help, -h                    显示此帮助信息");
    println!();
    println!("项目地址: https://github.com/yuta-sakata/mcs51-emulator");
}