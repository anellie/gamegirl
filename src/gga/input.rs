pub const INPUT_START: usize = 0x04000130;
pub const INPUT_END: usize = 0x04000133;

#[derive(Debug, Clone)]
pub struct Input {
    pub regs: [u8; 4],
}
