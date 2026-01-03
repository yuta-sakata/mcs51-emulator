mod cpu;

use cpu::CPU;
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

    // 初始化CPU
    let mut cpu = CPU::new(debug_mode);

    //HEX文件加载程序
    match cpu.load_hex_program(hex_file) {
        Ok(_) => println!("程序成功从 {} 加载", hex_file),
        Err(e) => {
            eprintln!("加载程序失败: {}", e);
            process::exit(1);
        }
    }
    
    loop {

        
        // 检查PC是否在有效范围内
        if (cpu.registers.pc as usize) >= cpu.rom.len() {
            println!("程序结杞: PC超出内存范围");
            break;
        }
        
        let pc = cpu.registers.pc;
        let opcode = cpu.rom[pc as usize];
        cpu.execute_instruction(opcode);
    }

    // 打印最终状态
    println!("CPU 状态：累加器 = {}, 程序计数器 = {}", cpu.registers.acc, cpu.registers.pc);
}