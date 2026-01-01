pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4 * 1024;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let emu = Emu::new();

        assert_eq!(emu.pc, 0x200);
        assert!(emu.v_reg.iter().all(|&v| v == 0));
        assert_eq!(emu.i_reg, 0);
        assert_eq!(emu.sp, 0);
        assert!(emu.stack.iter().all(|&v| v == 0));
        assert!(emu.keys.iter().all(|&k| !k));
        assert!(emu.screen.iter().all(|&pixel| !pixel));
    }

    #[test]
    fn test_ram_initially_zero() {
        let emu = Emu::new();

        assert!(emu.ram.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn test_screen_dimensions() {
        let emu = Emu::new();

        assert_eq!(emu.screen.len(), SCREEN_HEIGHT * SCREEN_WIDTH);
        assert_eq!(emu.screen.len(), 2048);
    }
}
