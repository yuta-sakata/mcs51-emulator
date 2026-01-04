// 循环检测器：跟踪PC历史，识别紧密循环并智能快进
// 这是一个性能优化工具，用于加速模拟器执行

pub struct LoopDetector {
    pc_history: Vec<u16>,       // 最近的PC历史（用于检测循环）
    pub loop_count: u32,            // 当前循环已执行次数
    pub loop_start: u16,            // 循环起始地址
    pub loop_end: u16,              // 循环结束地址
    skip_threshold: u32,        // 触发快进的阈值（循环次数）
    fast_forward_count: u32,    // 快进触发次数（用于检测重复快进）
    pub has_io_in_loop: bool,       // 循环中是否有I/O操作
    pub io_operation_count: u32,    // 循环中I/O操作计数
    instructions_in_loop: u32,  // 循环中的指令数
    last_fast_forward_time: u64, // 上次快进时的时钟周期
    pub same_loop_fast_forward_count: u32, // 同一循环快进次数（检测死循环）
    last_loop_start: u16,       // 上次循环的起始地址
    last_loop_end: u16,         // 上次循环的结束地址
}

impl LoopDetector {
    pub fn new() -> Self {
        LoopDetector {
            pc_history: Vec::with_capacity(100), // 存储最近100个PC
            loop_count: 0,                       // 当前循环计数
            loop_start: 0,                       // 循环起始地址
            loop_end: 0,                         // 循环结束地址
            skip_threshold: 100,                 // 循环超过100次才快进（观察期更长）
            fast_forward_count: 0,               // 重复快进计数
            has_io_in_loop: false,               // 默认无I/O
            io_operation_count: 0,               // I/O操作计数
            instructions_in_loop: 0,             // 循环指令数
            last_fast_forward_time: 0,           // 上次快进时间
            same_loop_fast_forward_count: 0,     // 同一循环快进次数
            last_loop_start: 0,                  // 上次循环起始
            last_loop_end: 0,                    // 上次循环结束
        }
    }

    // 记录PC并检测循环模式
    pub fn record_pc(&mut self, pc: u16) -> bool {
        // 检测简单的后向跳转（循环的标志）
        if self.pc_history.len() > 0 {
            let last_pc = self.pc_history[self.pc_history.len() - 1];

            // 检测后向跳转（pc <= last_pc），增大检测范围以捕获外层循环
            if pc <= last_pc && last_pc.saturating_sub(pc) < 50 {
                // 如果是同一个循环
                if self.loop_start == pc || self.loop_start == 0 {
                    if self.loop_start == 0 {
                        self.loop_start = pc;
                        self.loop_end = last_pc;
                    }
                    self.loop_count += 1;

                    // 达到阈值时触发快进
                    if self.loop_count >= self.skip_threshold {
                        return true;
                    }
                } else {
                    // 检测到新循环，重置
                    self.loop_start = pc;
                    self.loop_end = last_pc;
                    self.loop_count = 1;
                }
            } else if pc > last_pc.saturating_add(50) {
                // 跳出循环很远，重置
                if self.loop_count > 0 {
                    self.reset();
                }
            }
        }

        // 保持历史记录在合理大小
        if self.pc_history.len() > 50 {
            self.pc_history.remove(0);
        }
        self.pc_history.push(pc);
        false
    }

    pub fn reset(&mut self) {
        self.loop_count = 0;
    }

    // 检测是否为死循环（同一循环被快进太多次）
    pub fn is_deadlock(&self) -> bool {
        // 如果同一个循环被快进超过50次，认为是死循环
        self.same_loop_fast_forward_count > 50
    }

    // 检测是否为程序正常结束（单指令自跳转循环）
    pub fn is_program_end(&self) -> bool {
        // 如果循环开始和结束地址相同，说明是单指令循环（如 sjmp $）
        // 这通常表示程序正常结束
        self.loop_start == self.loop_end
    }

    // 在快进后调用，用于重置循环计数和检测死循环
    pub fn after_fast_forward(&mut self) {
        // 检查是否是同一个循环
        if self.last_loop_start == self.loop_start && self.last_loop_end == self.loop_end {
            self.same_loop_fast_forward_count += 1;
        } else {
            // 新循环，重置计数器
            self.same_loop_fast_forward_count = 1;
            self.last_loop_start = self.loop_start;
            self.last_loop_end = self.loop_end;
        }
        
        // 重置循环计数，准备下一轮检测
        self.loop_count = 0;
    }

    pub fn get_fast_forward_multiplier(&self) -> u64 {
        // 根据循环特征智能调整快进力度
        if self.has_io_in_loop {
            // 有I/O操作的循环：适度快进，保证输出频率真实
            // 快进到下一次I/O操作之前
            let cycles_per_loop = (self.instructions_in_loop as u64) * 12;
            let io_interval = if self.io_operation_count > 0 {
                cycles_per_loop * (self.loop_count as u64 / self.io_operation_count as u64).max(1)
            } else {
                cycles_per_loop * 10 // 快进10次循环
            };
            io_interval.min(100_000) // 最多快进10万周期
        } else {
            // 纯延时循环：大力快进，但保持时序准确
            // 根据嵌套深度加大快进力度
            if self.fast_forward_count > 10 {
                50_000_000 // 5000万周期 ≈ 4.17ms @ 12MHz
            } else if self.fast_forward_count > 5 {
                10_000_000 // 1000万周期 ≈ 0.83ms @ 12MHz
            } else if self.fast_forward_count > 2 {
                2_000_000  // 200万周期 ≈ 0.17ms @ 12MHz
            } else {
                500_000    // 50万周期 ≈ 0.04ms @ 12MHz
            }
        }
    }

    pub fn increment_fast_forward(&mut self) {
        self.fast_forward_count += 1;
    }

    // 记录循环中的I/O操作
    pub fn record_io_operation(&mut self) {
        self.io_operation_count += 1;
        self.has_io_in_loop = true;
    }

    // 记录循环中的指令数
    pub fn set_loop_size(&mut self, size: u32) {
        self.instructions_in_loop = size;
    }
}
