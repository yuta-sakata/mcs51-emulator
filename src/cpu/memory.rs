use super::CPU;

impl CPU {
    pub fn fetch_next_byte(&mut self) -> u8 {
        if self.registers.pc as usize >= self.rom.len() {
            println!("错误: 尝试从超出内存范围的地址读取");
            return 0;
        }
        let byte = self.rom[self.registers.pc as usize];
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }
}