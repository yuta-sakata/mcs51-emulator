pub struct Registers {
    pub acc: u8,   // 累加器 A
    pub b: u8,     // 寄存器 B
    pub pc: u16,   // 程序计数器
    pub sp: u8,    // 堆栈指针
    pub dptr: u16, // 数据指针 DPTR
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            acc: 0,
            b: 0,
            pc: 0,
            sp: 7,
            dptr: 0,
        }
    }
}