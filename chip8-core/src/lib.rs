pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4 * 1024;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[allow(dead_code)]
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

impl Default for Emu {
    fn default() -> Self {
        Self::new()
    }
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
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
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    #[allow(dead_code)]
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    #[allow(dead_code)]
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode
        // Execute
        self.execute(op);
        self.tick_timers();
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        // Instructions implementation based on http://devernay.free.fr/hacks/chip8/C8TECH10.HTM and https://aquova.net/emudev/chip8/
        #[allow(clippy::match_single_binding)]
        match (digit1, digit2, digit3, digit4) {
            // 0000 -- NOP
            (0, 0, 0, 0) => (),
            // 00E0 -- CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // 00EE -- RET
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            // 1NNN -- Jump
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            }
            // 2NNN -- Call
            (2, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // 3XNN -- Skip new if VX = NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // BNNN -- Jump to V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            // 4XNN -- Skip next instruction if VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = op & 0x00FF;

                if (self.v_reg[x] as u16) != nn {
                    self.pc += 2;
                }
            }
            // 5XY0 -- Skip next instruction if VX = VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // 9XY0 -- Skip next instruction if Vx != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // EX9E -- Skip next instruction if key with the value of VX is pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let key_val = self.v_reg[x] as usize;

                if key_val < NUM_KEYS && self.keys[key_val] {
                    self.pc += 2;
                }
            }
            // FX0A -- Wait for key
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;

                if let Some(key) = (0..self.keys.len()).find(|&i| self.keys[i]) {
                    self.v_reg[x] = key as u8;
                } else {
                    self.pc -= 2; // Loop until a key is pressed
                }
            }
            // 6XKK -- Set VX = KK
            (6, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;

                self.v_reg[x] = kk;
            }
            // 7XKK -- Set VX = VX + KK
            (7, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;

                self.v_reg[x] = self.v_reg[x].wrapping_add(kk);
            }
            // 8XY0 -- Set VX = VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] = self.v_reg[y];
            }
            // ANNN -- I = NNN
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    #[allow(dead_code)]
    fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                //BEEP
                // TODO: Emit beep sound
            }
            self.st -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let emu = Emu::new();

        assert_eq!(emu.pc, START_ADDR);
        assert!(emu.v_reg.iter().all(|&v| v == 0));
        assert_eq!(emu.i_reg, 0);
        assert_eq!(emu.sp, 0);
        assert!(emu.stack.iter().all(|&v| v == 0));
        assert!(emu.keys.iter().all(|&k| !k));
        assert!(emu.screen.iter().all(|&pixel| !pixel));
    }

    #[test]
    fn test_ram_head_contains_fontset() {
        let emu = Emu::new();

        assert_eq!(emu.ram[..FONTSET_SIZE], FONTSET);
    }

    #[test]
    fn test_ram_tail_initially_zero() {
        let emu = Emu::new();

        assert!(emu.ram[FONTSET_SIZE..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn test_screen_dimensions() {
        let emu = Emu::new();

        assert_eq!(emu.screen.len(), SCREEN_HEIGHT * SCREEN_WIDTH);
        assert_eq!(emu.screen.len(), 2048);
    }

    #[test]
    fn test_reset() {
        let mut emu = Emu::new();

        // Modify state
        emu.pc = 0x300;
        emu.v_reg[0] = 42;
        emu.dt = 10;
        emu.st = 5;
        emu.screen[0] = true;
        emu.ram[100] = 0xFF;

        // Reset state
        emu.reset();

        assert_eq!(emu.pc, START_ADDR);
        assert!(emu.v_reg.iter().all(|&v| v == 0));
        assert_eq!(emu.dt, 0);
        assert_eq!(emu.st, 0);
        assert!(emu.screen.iter().all(|&pixel| !pixel));
        assert_eq!(emu.ram[..FONTSET_SIZE], FONTSET); // fontset still loaded
        assert!(emu.ram[FONTSET_SIZE..].iter().all(|&b| b == 0)); // rest of the ram still zero
    }

    #[test]
    fn test_fetch() {
        let mut emu = Emu::new();

        emu.ram[0x200] = 0xAB;
        emu.ram[0x201] = 0xCD;

        let op = emu.fetch();

        assert_eq!(op, 0xABCD);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_push_and_pop() {
        let mut emu = Emu::new();

        emu.push(0x123);
        emu.push(0x456);
        emu.push(0x789);

        assert_eq!(emu.sp, 3);

        assert_eq!(emu.pop(), 0x789);
        assert_eq!(emu.pop(), 0x456);
        assert_eq!(emu.pop(), 0x123);

        assert_eq!(emu.sp, 0);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_pop_underflow() {
        let mut emu = Emu::new();
        emu.pop(); // should panick (sp = 0, sp -= 1 -> underflow)
    }

    #[test]
    fn test_tick_timers() {
        let mut emu = Emu::new();

        emu.dt = 5;
        emu.st = 3;

        emu.tick_timers();
        assert_eq!(emu.dt, 4);
        assert_eq!(emu.st, 2);

        emu.dt = 1;
        emu.st = 1;

        emu.tick_timers();
        assert_eq!(emu.dt, 0);
        assert_eq!(emu.st, 0); // st == 1 -> beep, then st-=1 -> 0

        emu.dt = 0;
        emu.st = 0;
        emu.tick_timers();
        assert_eq!(emu.dt, 0);
        assert_eq!(emu.st, 0);
    }

    #[test]
    #[should_panic(expected = "Unimplemented opcode")]
    fn test_execute_unimplemented() {
        let mut emu = Emu::new();

        // Some unimplemented opcode
        let op: u16 = 0xFFFF;

        // Expected panic with specific communicate
        emu.execute(op);
    }

    // Opcodes

    #[test]
    fn test_opcode_0000_nop() {
        let mut emu = Emu::new();
        emu.pc = 0x300;

        emu.execute(0x0000);

        assert_eq!(emu.pc, 0x300);
    }

    #[test]
    fn test_opcode_00e0_cls() {
        let mut emu = Emu::new();

        emu.screen[0] = true;
        emu.screen[100] = true;
        emu.screen[2047] = true;

        emu.execute(0x00E0);

        assert!(emu.screen.iter().all(|&pixel| !pixel));
        assert_eq!(emu.pc, START_ADDR);
    }

    #[test]
    fn test_opcode_00ee_ret() {
        let mut emu = Emu::new();

        emu.push(0x456);
        emu.pc = 0x300;

        emu.execute(0x00EE);

        assert_eq!(emu.pc, 0x456);
        assert_eq!(emu.sp, 0);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_opcode_00ee_ret_empty_stack() {
        let mut emu = Emu::new();
        emu.execute(0x00EE);
    }

    #[test]
    fn test_opcode_1nnn_jump() {
        let mut emu = Emu::new();

        emu.execute(0x1ABC);

        assert_eq!(emu.pc, 0xABC);
    }

    #[test]
    fn test_opcode_2nnn_call() {
        let mut emu = Emu::new();
        emu.pc = 0x300;

        emu.execute(0x2DEF);

        assert_eq!(emu.pc, 0xDEF);
        assert_eq!(emu.sp, 1);
        assert_eq!(emu.pop(), 0x300);
    }

    #[test]
    fn test_opcode_bnnn_jump_v0_offset() {
        let mut emu = Emu::new();
        emu.v_reg[0] = 0x50;

        emu.execute(0xB123);

        assert_eq!(emu.pc, 0x173);
    }

    #[test]
    fn test_opcode_bnnn_jump_v0_offset_zero() {
        let mut emu = Emu::new();
        emu.v_reg[0] = 0x00;

        emu.execute(0xB000);

        assert_eq!(emu.pc, 0x000);
    }

    #[test]
    fn test_opcode_4xnn_skip_when_vx_not_equal_nn() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x4B;
        emu.ram[0x201] = 0x77;

        emu.v_reg[0xB] = 0x42;

        emu.tick();

        assert_eq!(emu.pc, 0x200 + 4); // +2 (fetch) and +2 (skip)
    }

    #[test]
    fn test_opcode_4xnn_no_skip_when_vs_equal_nn() {
        let mut emu = Emu::new();
        emu.pc = 0x300;
        emu.v_reg[0x4] = 0xAB;

        emu.execute(0x45AB);

        assert_eq!(emu.pc, 0x300 + 2); // normal jump
    }

    #[test]
    fn test_opcode_5xy0_skip_when_vx_equal_vy() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x51;
        emu.ram[0x201] = 0xA0;

        emu.v_reg[0x1] = 0xFE;
        emu.v_reg[0xA] = 0xFE;

        emu.tick();

        assert_eq!(emu.pc, 0x200 + 4); // +2 (fetch), +2 (skip)
    }

    #[test]
    fn test_opcode_5xy0_skip_when_vx_not_equal_vy() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x51;
        emu.ram[0x201] = 0xA0;

        emu.v_reg[0x1] = 0x21;
        emu.v_reg[0xA] = 0x37;

        emu.tick();

        assert_eq!(emu.pc, 0x200 + 2);
    }

    #[test]
    fn test_opcode_9xy0_no_skip_when_vx_equal_vy() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x9B;
        emu.ram[0x201] = 0xC0;

        emu.v_reg[0xB] = 0x21;
        emu.v_reg[0xC] = 0x21;

        emu.tick();

        assert_eq!(emu.pc, 0x200 + 2);
    }

    #[test]
    fn test_opcode_9xy0_skip_when_vx_not_equal_vy() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x99;
        emu.ram[0x201] = 0x80;

        emu.v_reg[0x9] = 0x21;
        emu.v_reg[0x8] = 0x37;

        emu.tick();

        assert_eq!(emu.pc, 0x200 + 4);
    }

    #[test]
    fn test_opcode_ex9e_skip_when_key_pressed() {
        let mut emu = Emu::new();
        emu.pc = 0x400;

        emu.ram[0x400] = 0xE4;
        emu.ram[0x401] = 0x9E;

        emu.v_reg[0x4] = 0x9;
        emu.keys[9] = true;

        emu.tick();

        assert_eq!(emu.pc, 0x400 + 4);
    }

    #[test]
    fn test_opcode_ex9e_no_skip_when_key_not_pressed() {
        let mut emu = Emu::new();
        emu.pc = 0x300;

        emu.ram[0x300] = 0xED;
        emu.ram[0x301] = 0x9E;

        emu.v_reg[0xD] = 0x2;
        emu.keys[2] = false;

        emu.tick();

        assert_eq!(emu.pc, 0x300 + 2);
    }

    #[test]
    fn test_opcode_ex9e_no_skip_when_key_value_out_of_range() {
        let mut emu = Emu::new();
        emu.pc = 0x400;

        emu.ram[0x400] = 0xE1;
        emu.ram[0x401] = 0x9E;

        emu.v_reg[0x1] = 0xFF; // out of range 0-15

        emu.tick();

        assert_eq!(emu.pc, 0x400 + 2);
    }

    #[test]
    fn test_opcode_fx0a_stores_key_when_pressed() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0xF3;
        emu.ram[0x201] = 0x0A;

        emu.keys[0xC] = true;

        emu.tick();

        assert_eq!(emu.v_reg[0x3], 0xC);
        assert_eq!(emu.pc, 0x200 + 2);
    }

    #[test]
    fn test_opcode_fx0a_repeats_when_no_key_pressed() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0xF8;
        emu.ram[0x201] = 0x0A;

        emu.tick();

        assert_eq!(emu.pc, 0x200); // reversed by 2
    }

    #[test]
    fn test_opcode_fx0a_takes_first_pressed_key() {
        let mut emu = Emu::new();
        emu.keys[5] = true;
        emu.keys[11] = true;

        emu.execute(0xFE0A); // store to V14

        assert_eq!(emu.v_reg[0xE], 5); // takes key with lowest index
    }

    #[test]
    fn test_opcode_6xkk_puts_value_into_register() {
        let mut emu = Emu::new();

        emu.pc = 0x200;

        emu.ram[0x200] = 0x61;
        emu.ram[0x201] = 0x42;

        emu.tick();

        assert_eq!(emu.v_reg[1], 0x42);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_6xkk_load_zero() {
        let mut emu = Emu::new();

        emu.pc = 0x200;

        emu.ram[0x200] = 0x60;
        emu.ram[0x201] = 0x00;

        emu.tick();

        assert_eq!(emu.v_reg[0x0], 0x00);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_6xkk_load_max_value() {
        let mut emu = Emu::new();

        emu.pc = 0x300;

        emu.ram[0x300] = 0x6F;
        emu.ram[0x301] = 0xFF;

        emu.tick();

        assert_eq!(emu.v_reg[0xF], 0xFF);
        assert_eq!(emu.pc, 0x302);
    }

    #[test]
    fn test_opcode_6xkk_overwrite_previous_value() {
        let mut emu = Emu::new();

        emu.pc = 0x300;

        emu.v_reg[0xF] = 0x42;
        assert_eq!(emu.v_reg[0xF], 0x42);

        emu.ram[0x300] = 0x6F;
        emu.ram[0x301] = 0x33;

        emu.tick();

        assert_eq!(emu.v_reg[0xF], 0x33);
        assert_eq!(emu.pc, 0x302);
    }

    #[test]
    fn test_opcode_6xkk_multiple_in_sequence_same_register() {
        let mut emu = Emu::new();

        emu.pc = 0x300;

        emu.ram[0x300] = 0x64;
        emu.ram[0x301] = 0x33;
        emu.tick();
        assert_eq!(emu.v_reg[0x4], 0x33);
        assert_eq!(emu.pc, 0x302);

        emu.ram[0x302] = 0x64;
        emu.ram[0x303] = 0x42;
        emu.tick();
        assert_eq!(emu.v_reg[0x4], 0x42);
        assert_eq!(emu.pc, 0x304);

        emu.ram[0x304] = 0x64;
        emu.ram[0x305] = 0x21;
        emu.tick();
        assert_eq!(emu.v_reg[0x4], 0x21);
        assert_eq!(emu.pc, 0x306);
    }
    #[test]
    fn test_opcode_6xkk_multiple_in_sequence_different_registers() {
        let mut emu = Emu::new();

        emu.pc = 0x300;

        emu.ram[0x300] = 0x64;
        emu.ram[0x301] = 0x33;
        emu.tick();
        assert_eq!(emu.v_reg[0x4], 0x33);
        assert_eq!(emu.pc, 0x302);

        emu.ram[0x302] = 0x62;
        emu.ram[0x303] = 0x42;
        emu.tick();
        assert_eq!(emu.v_reg[0x2], 0x42);
        assert_eq!(emu.pc, 0x304);

        emu.ram[0x304] = 0x6A;
        emu.ram[0x305] = 0x21;
        emu.tick();
        assert_eq!(emu.v_reg[0xA], 0x21);
        assert_eq!(emu.pc, 0x306);
    }

    #[test]
    fn test_opcode_7xkk_add_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x73;
        emu.ram[0x201] = 0x45;
        emu.v_reg[0x3] = 0x20;

        emu.tick();

        assert_eq!(emu.v_reg[0x3], 0x20 + 0x45);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_7xkk_add_zero() {
        let mut emu = Emu::new();
        emu.pc = 0x300;
        emu.ram[0x300] = 0x70;
        emu.ram[0x301] = 0x00;
        emu.v_reg[0x0] = 0xAB;

        emu.tick();

        assert_eq!(emu.v_reg[0x0], 0xAB);
        assert_eq!(emu.pc, 0x302);
    }

    #[test]
    fn test_opcode_7xkk_add_max_value() {
        let mut emu = Emu::new();
        emu.pc = 0x400;
        emu.ram[0x400] = 0x7F;
        emu.ram[0x401] = 0xFF;
        emu.v_reg[0xF] = 0x00;

        emu.tick();

        assert_eq!(emu.v_reg[0xF], 0xFF);
        assert_eq!(emu.pc, 0x402);
    }

    #[test]
    fn test_opcode_7xkk_add_overflow() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x7A;
        emu.ram[0x201] = 0x80;
        emu.v_reg[0xA] = 0xFF;

        emu.tick();

        assert_eq!(emu.v_reg[0xA], 0x7F); // 0x017F → 0x7F (overflow, 8-bit wrap)
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_7xkk_add_multiple_times() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.v_reg[0x5] = 0x10;

        emu.ram[0x200] = 0x75;
        emu.ram[0x201] = 0x20;
        emu.ram[0x202] = 0x75;
        emu.ram[0x203] = 0x30;
        emu.ram[0x204] = 0x75;
        emu.ram[0x205] = 0x40;

        emu.tick();
        assert_eq!(emu.v_reg[0x5], 0x30);

        emu.tick();
        assert_eq!(emu.v_reg[0x5], 0x60);

        emu.tick();
        assert_eq!(emu.v_reg[0x5], 0xA0);
        assert_eq!(emu.pc, 0x206);
    }
}
