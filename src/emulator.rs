// 模拟器包装层 - 负责执行优化、调试、性能统计等非硬件功能
use crate::cpu::CPU;
use crate::loop_detector::LoopDetector;

pub struct Emulator {
    pub cpu: CPU,
    pub debug: bool,                    // 调试模式
    pub clock_cycles: u64,              // 时钟周期计数
    pub loop_detector: LoopDetector,    // 循环检测器
    pub delay_skip_counter: u32,        // 延迟跳过计数器（用于优化特定函数）
    pub instruction_count: u64,         // 总指令执行计数
    pub is_halted: bool,                // 是否已停机（死循环或错误）
}

impl Emulator {
    pub fn new(debug: bool) -> Self {
        Emulator {
            cpu: CPU::new(),
            debug,
            clock_cycles: 0,
            loop_detector: LoopDetector::new(),
            delay_skip_counter: 0,
            instruction_count: 0,
            is_halted: false,
        }
    }

    // 执行单条指令（带优化和调试）
    pub fn execute_instruction(&mut self, opcode: u8) {
        // 检查是否已停机
        if self.is_halted {
            return;
        }

        // 指令计数
        self.instruction_count += 1;

        // 保存当前 PC 用于调试输出
        let pc_before = self.cpu.registers.pc;

        // 循环检测：如果检测到紧密循环超过阈值，快进
        if self.loop_detector.record_pc(pc_before) {
            self.loop_detector.increment_fast_forward();
            
            // 计算循环大小
            let loop_size = if self.loop_detector.loop_end >= self.loop_detector.loop_start {
                ((self.loop_detector.loop_end - self.loop_detector.loop_start) / 2) as u32 // 估算指令数
            } else {
                1
            };
            self.loop_detector.set_loop_size(loop_size.max(1));
            
            let multiplier = self.loop_detector.get_fast_forward_multiplier();
            let has_io = self.loop_detector.has_io_in_loop;

            // 只在调试模式且非单指令死循环时输出快进信息
            // 或者在单指令死循环的前几次快进时输出
            let should_print = self.debug && (
                !self.loop_detector.is_program_end() || 
                self.loop_detector.same_loop_fast_forward_count < 3
            );

            if should_print {
                let loop_type = if has_io { "I/O循环" } else { "纯延时循环" };
                println!(
                    "\n[LOOP FAST-FORWARD] 检测到{} ({:#06x}-{:#06x})，已执行 {} 次，I/O次数: {}，快进 {} 个周期 ({:.2} ms @ 12MHz)...",
                    loop_type,
                    self.loop_detector.loop_start,
                    self.loop_detector.loop_end,
                    self.loop_detector.loop_count,
                    self.loop_detector.io_operation_count,
                    multiplier,
                    multiplier as f64 / 12_000_000.0 * 1000.0
                );
            }

            // 快进：增加大量时钟周期
            self.clock_cycles += multiplier;

            // 如果是单指令等待循环（loop_size <= 1），不要修改PC，让它继续执行以便中断能触发
            // 否则跳到循环结束之后继续
            if loop_size > 1 {
                self.cpu.registers.pc = self.loop_detector.loop_end.wrapping_add(1);
            }

            // 快进后重置并检测死循环
            self.loop_detector.after_fast_forward();
            
            // 检测死循环
            if self.loop_detector.is_deadlock() {
                // 判断是程序正常结束还是真正的死循环
                if self.loop_detector.is_program_end() {
                    // 单指令死循环，通常是程序正常结束（如 sjmp $）
                    // 继续运行但不再输出快进信息（可能在等待中断）
                    if self.debug && self.loop_detector.same_loop_fast_forward_count == 51 {
                        // 只在第一次检测到时输出一次
                        println!("\n[信息] 程序到达结束点 {:#06x} (进入待机循环)", 
                            self.loop_detector.loop_start);
                    }
                    // 不停机，继续运行
                } else {
                    // 真正的死循环错误
                    println!("\n[警告] 检测到死循环在 {:#06x}-{:#06x}，程序可能在等待永远不会发生的事件（如中断或外部输入）", 
                        self.loop_detector.loop_start, 
                        self.loop_detector.loop_end);
                    println!("提示: 程序在地址 {:#06x} 处陷入无限等待", pc_before);
                    self.is_halted = true;
                }
            }

            return;
        }

        // 每条指令消耗12个时钟周期（简化）
        self.clock_cycles += 12;

        // 在 debug 模式下，打印 [时钟周期][地址] 前缀
        if self.debug {
            print!("[{}][{:#06x}] ", self.clock_cycles, pc_before);
        }

        // 执行真实的CPU指令
        self.cpu.execute_instruction(opcode, self.debug, &mut self.delay_skip_counter);
    }

    // 执行带调试信息的端口写入
    pub fn write_sfr(&mut self, addr: u8, value: u8) {
        if self.debug && addr == 0x90 {
            // P1端口输出 (串口数据输出)
            if value >= 0x20 && value <= 0x7E {
                print!("[串口输出] 字符: {} (ASCII {:#04x})", value as char, value);
            } else {
                print!("[串口输出] 数据: {:#04x}", value);
            }
        }
        self.cpu.write_sfr(addr, value);
    }
}
