mod cpu;
mod emulator;
mod loop_detector;

use emulator::Emulator;
use std::env;
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
    if args.len() < 2 {
        eprintln!("用法: {} <程序文件> [--debug]", args[0]);
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