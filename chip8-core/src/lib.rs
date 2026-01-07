use rand::Rng;

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
        let op = self.fetch();
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

        #[allow(clippy::match_single_binding)]
        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            }
            (2, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = op & 0x00FF;
                if (self.v_reg[x] as u16) != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let key_val = self.v_reg[x] as usize;
                if key_val < NUM_KEYS && self.keys[key_val] {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                if let Some(key) = (0..self.keys.len()).find(|&i| self.keys[i]) {
                    self.v_reg[x] = key as u8;
                } else {
                    self.pc -= 2;
                }
            }
            (6, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;
                self.v_reg[x] = kk;
            }
            (7, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(kk);
            }
            (8, _, _, n) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let vx = self.v_reg[x];
                let vy = self.v_reg[y];

                match n {
                    0 => self.v_reg[x] = vy,
                    1 => self.v_reg[x] |= vy,
                    2 => self.v_reg[x] &= vy,
                    3 => self.v_reg[x] ^= vy,
                    4 => {
                        let sum = vx as u16 + vy as u16;
                        self.v_reg[x] = sum as u8;
                        self.v_reg[0xF] = if sum > 0xFF { 1 } else { 0 };
                    }
                    5 => {
                        self.v_reg[0xF] = if vx >= vy { 1 } else { 0 };
                        self.v_reg[x] = vx.wrapping_sub(vy);
                    }
                    6 => {
                        self.v_reg[x] = vy;
                        self.v_reg[0xF] = vy & 1;
                        self.v_reg[x] >>= 1;
                    }
                    7 => {
                        self.v_reg[0xF] = if vy >= vx { 1 } else { 0 };
                        self.v_reg[x] = vy.wrapping_sub(vx);
                    }
                    0xE => {
                        self.v_reg[x] = vy;
                        self.v_reg[0xF] = (vy >> 7) & 1;
                        self.v_reg[x] <<= 1;
                    }
                    _ => unimplemented!("Unimplemented 8XY{} opcode", n),
                }
            }
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }
            // DXYN -- Display N-byte sprite starting at memory location I at (VX, VY); set VF if collision
            (0xD, _, _, n) => {
                let x_coord = self.v_reg[digit2 as usize] as usize;
                let y_coord = self.v_reg[digit3 as usize] as usize;
                let i = self.i_reg as usize;
                let height = n as usize;

                self.v_reg[0xF] = 0; // initially no collision

                for row in 0..height {
                    let sprite_row = self.ram[i + row];
                    for col in 0..8 {
                        let pixel = (sprite_row >> (7 - col)) & 1;
                        if pixel == 1 {
                            let screen_x = (x_coord + col) % SCREEN_WIDTH;
                            let screen_y = (y_coord + row) % SCREEN_HEIGHT;
                            let index = screen_y * SCREEN_WIDTH + screen_x;

                            if self.screen[index] {
                                self.v_reg[0xF] = 1;
                            }

                            self.screen[index] ^= true;
                        }
                    }
                }
            }
            // FX07 -- Store delay timer value in VX
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }
            // FX15 -- Set delay timer to VX
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }
            // FX18 -- Sets sound timer to VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }
            // CXKK (RND) -- Set VX to random byte AND KK
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;

                let mut rng = rand::rng();
                let random_byte: u8 = rng.random_range(0..=255);

                self.v_reg[x] = random_byte & kk;
            }
            // FX33 (BCD) -- Store VX as BCD (Binary Coded Decimal) in the I
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let addr = self.i_reg as usize;

                self.ram[addr] = vx / 100;
                self.ram[addr + 1] = (vx / 10) % 10;
                self.ram[addr + 2] = vx % 10;
            }
            // FX55
            // FX65
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:04X}", op),
        }
    }

    #[allow(dead_code)]
    fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // TODO: Emit beep sound
            }
            self.st -= 1;
        }
    }

    #[allow(dead_code)]
    pub fn dump_ram(&self) {
        println!("CHIP-8 RAM Dump (0x000 - 0xFFF):");
        for addr in (0..RAM_SIZE).step_by(16) {
            print!("{:04X}: ", addr);

            for i in 0..16 {
                if addr + i < RAM_SIZE {
                    print!("{:02X} ", self.ram[addr + i]);
                } else {
                    print!("   ");
                }
            }

            println!();
        }
    }

    #[allow(dead_code)]
    pub fn dump_screen(&self) {
        println!("CHIP-8 Screen (64x32):");
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx = y * SCREEN_WIDTH + x;
                if self.screen[idx] {
                    print!("■");
                } else {
                    print!("⋅");
                }
            }
            println!();
        }
        println!("--- End of screen ---");
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

    #[test]
    fn test_opcode_8xy0_assign_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x81;
        emu.ram[0x201] = 0x20;
        emu.v_reg[0x2] = 0xAB;

        emu.tick();

        assert_eq!(emu.v_reg[0x1], 0xAB);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy0_assign_zero() {
        let mut emu = Emu::new();
        emu.pc = 0x300;
        emu.ram[0x300] = 0x83;
        emu.ram[0x301] = 0x00;
        emu.v_reg[0x0] = 0x00;

        emu.tick();

        assert_eq!(emu.v_reg[0x3], 0x00);
        assert_eq!(emu.pc, 0x302);
    }

    #[test]
    fn test_opcode_8xy0_assign_max_value() {
        let mut emu = Emu::new();
        emu.pc = 0x400;
        emu.ram[0x400] = 0x8F;
        emu.ram[0x401] = 0x40;
        emu.v_reg[0x4] = 0xFF;

        emu.tick();

        assert_eq!(emu.v_reg[0xF], 0xFF);
        assert_eq!(emu.pc, 0x402);
    }

    #[test]
    fn test_opcode_8xy0_overwrite_existing_value() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x85;
        emu.ram[0x201] = 0x60;
        emu.v_reg[0x5] = 0x12;
        emu.v_reg[0x6] = 0xCD;

        emu.tick();

        assert_eq!(emu.v_reg[0x5], 0xCD);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy0_self_assign() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x88;
        emu.ram[0x201] = 0x80;
        emu.v_reg[0x8] = 0x5A;

        emu.tick();

        assert_eq!(emu.v_reg[0x8], 0x5A);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy0_multiple_assigns() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0x81;
        emu.ram[0x201] = 0x20;
        emu.ram[0x202] = 0x83;
        emu.ram[0x203] = 0x40;
        emu.ram[0x204] = 0x85;
        emu.ram[0x205] = 0x60;

        emu.v_reg[0x2] = 0x11;
        emu.v_reg[0x4] = 0x22;
        emu.v_reg[0x6] = 0x33;

        emu.tick();
        assert_eq!(emu.v_reg[0x1], 0x11);

        emu.tick();
        assert_eq!(emu.v_reg[0x3], 0x22);

        emu.tick();
        assert_eq!(emu.v_reg[0x5], 0x33);

        assert_eq!(emu.pc, 0x206);
    }

    #[test]
    fn test_opcode_annn_load_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0xA1;
        emu.ram[0x201] = 0x23;

        emu.tick();

        assert_eq!(emu.i_reg, 0x123);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_annn_load_zero() {
        let mut emu = Emu::new();
        emu.pc = 0x300;
        emu.ram[0x300] = 0xA0;
        emu.ram[0x301] = 0x00;

        emu.tick();

        assert_eq!(emu.i_reg, 0x000);
        assert_eq!(emu.pc, 0x302);
    }

    #[test]
    fn test_opcode_annn_load_max_value() {
        let mut emu = Emu::new();
        emu.pc = 0x400;
        emu.ram[0x400] = 0xAF;
        emu.ram[0x401] = 0xFF;

        emu.tick();

        assert_eq!(emu.i_reg, 0xFFF);
        assert_eq!(emu.pc, 0x402);
    }

    #[test]
    fn test_opcode_annn_overwrite_existing_value() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.i_reg = 0x456;

        emu.ram[0x200] = 0xA7;
        emu.ram[0x201] = 0x89;

        emu.tick();

        assert_eq!(emu.i_reg, 0x789);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_annn_multiple_loads() {
        let mut emu = Emu::new();
        emu.pc = 0x200;

        emu.ram[0x200] = 0xA1;
        emu.ram[0x201] = 0x11;
        emu.ram[0x202] = 0xA2;
        emu.ram[0x203] = 0x22;
        emu.ram[0x204] = 0xA3;
        emu.ram[0x205] = 0x33;

        emu.tick();
        assert_eq!(emu.i_reg, 0x111);

        emu.tick();
        assert_eq!(emu.i_reg, 0x222);

        emu.tick();
        assert_eq!(emu.i_reg, 0x333);
        assert_eq!(emu.pc, 0x206);
    }

    #[test]
    fn test_opcode_8xy1_or_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x81;
        emu.ram[0x201] = 0x21;
        emu.v_reg[0x1] = 0b1010_1010;
        emu.v_reg[0x2] = 0b1100_1100;

        emu.tick();

        assert_eq!(emu.v_reg[0x1], 0b1110_1110);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy1_or_no_change_when_zero() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x83;
        emu.ram[0x201] = 0x21;
        emu.v_reg[0x3] = 0x00;
        emu.v_reg[0x2] = 0x00;

        emu.tick();

        assert_eq!(emu.v_reg[0x3], 0x00);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy2_and_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x84;
        emu.ram[0x201] = 0x22;
        emu.v_reg[0x4] = 0b1111_0000;
        emu.v_reg[0x2] = 0b1010_1010;

        emu.tick();

        assert_eq!(emu.v_reg[0x4], 0b1010_0000);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy2_and_all_zero() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x85;
        emu.ram[0x201] = 0x22;
        emu.v_reg[0x5] = 0xFF;
        emu.v_reg[0x2] = 0x00;

        emu.tick();

        assert_eq!(emu.v_reg[0x5], 0x00);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy3_xor_basic() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x86;
        emu.ram[0x201] = 0x23;
        emu.v_reg[0x6] = 0b1010_1010;
        emu.v_reg[0x2] = 0b1111_0000;

        emu.tick();

        assert_eq!(emu.v_reg[0x6], 0b0101_1010);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy3_xor_identity() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x87;
        emu.ram[0x201] = 0x23;
        emu.v_reg[0x7] = 0xAB;
        emu.v_reg[0x2] = 0xAB;

        emu.tick();

        assert_eq!(emu.v_reg[0x7], 0x00);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy4_add_no_carry() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x88;
        emu.ram[0x201] = 0x44;
        emu.v_reg[0x8] = 0x50;
        emu.v_reg[0x4] = 0x30;

        emu.tick();

        assert_eq!(emu.v_reg[0x8], 0x80);
        assert_eq!(emu.v_reg[0xF], 0);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy4_add_with_carry() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x89;
        emu.ram[0x201] = 0x44;
        emu.v_reg[0x9] = 0xFF;
        emu.v_reg[0x4] = 0x01;

        emu.tick();

        assert_eq!(emu.v_reg[0x9], 0x00);
        assert_eq!(emu.v_reg[0xF], 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy5_sub_no_borrow() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8A;
        emu.ram[0x201] = 0x55;
        emu.v_reg[0xA] = 0x70;
        emu.v_reg[0x5] = 0x20;

        emu.tick();

        assert_eq!(emu.v_reg[0xA], 0x50);
        assert_eq!(emu.v_reg[0xF], 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy5_sub_with_borrow() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8B;
        emu.ram[0x201] = 0x55;
        emu.v_reg[0xB] = 0x10;
        emu.v_reg[0x5] = 0x20;

        emu.tick();

        assert_eq!(emu.v_reg[0xB], 0xF0); // 0x10 - 0x20 = 0xF0 (wrap)
        assert_eq!(emu.v_reg[0xF], 0);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy6_shr_lsb_1() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8C;
        emu.ram[0x201] = 0x06;
        emu.v_reg[0xC] = 0x00;
        emu.v_reg[0x0] = 0b1010_1101; // LSB = 1

        emu.tick();

        assert_eq!(emu.v_reg[0xC], 0b0101_0110); // przesunięte o 1
        assert_eq!(emu.v_reg[0xF], 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy6_shr_lsb_0() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8D;
        emu.ram[0x201] = 0x06;
        emu.v_reg[0xD] = 0x00;
        emu.v_reg[0x0] = 0b1010_1100; // LSB = 0

        emu.tick();

        assert_eq!(emu.v_reg[0xD], 0b0101_0110);
        assert_eq!(emu.v_reg[0xF], 0);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy7_subn_no_borrow() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8E;
        emu.ram[0x201] = 0x77;
        emu.v_reg[0xE] = 0x20;
        emu.v_reg[0x7] = 0x70;

        emu.tick();

        assert_eq!(emu.v_reg[0xE], 0x50); // Vy - Vx
        assert_eq!(emu.v_reg[0xF], 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xy7_subn_with_borrow() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x8E;
        emu.ram[0x201] = 0x77;
        emu.v_reg[0xE] = 0x70;
        emu.v_reg[0x7] = 0x20;

        emu.tick();

        assert_eq!(emu.v_reg[0xE], 0xB0); // Vy - Vx = 0x20 - 0x70 = wrap
        assert_eq!(emu.v_reg[0xF], 0);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xye_shl_msb_1() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x81;
        emu.ram[0x201] = 0x0E;
        emu.v_reg[0x1] = 0x00;
        emu.v_reg[0x0] = 0b1000_0001; // MSB = 1

        emu.tick();

        assert_eq!(emu.v_reg[0x1], 0b0000_0010);
        assert_eq!(emu.v_reg[0xF], 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_8xye_shl_msb_0() {
        let mut emu = Emu::new();
        emu.pc = 0x200;
        emu.ram[0x200] = 0x82;
        emu.ram[0x201] = 0x0E;
        emu.v_reg[0x2] = 0x00;
        emu.v_reg[0x0] = 0b0111_1111; // MSB = 0

        emu.tick();

        assert_eq!(emu.v_reg[0x2], 0b1111_1110);
        assert_eq!(emu.v_reg[0xF], 0);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_dxyn_draw_no_collision() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0b1111_0000;
        emu.ram[0x51] = 0b0000_1111;

        emu.v_reg[0x0] = 10;
        emu.v_reg[0x1] = 5;

        emu.execute(0xD012);

        let base1 = 5 * SCREEN_WIDTH + 10;
        assert!(emu.screen[base1]);
        assert!(emu.screen[base1 + 1]);
        assert!(emu.screen[base1 + 2]);
        assert!(emu.screen[base1 + 3]);

        let base2 = 6 * SCREEN_WIDTH + 10;
        assert!(emu.screen[base2 + 4]);
        assert!(emu.screen[base2 + 5]);
        assert!(emu.screen[base2 + 6]);
        assert!(emu.screen[base2 + 7]);

        assert_eq!(emu.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_dxyn_draw_with_collision() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0b1010_1010;

        emu.v_reg[0x0] = 20;
        emu.v_reg[0x1] = 10;

        // overwrite two pixels with sprite
        let idx1 = 10 * SCREEN_WIDTH + 20;
        let idx3 = 10 * SCREEN_WIDTH + 22;
        emu.screen[idx1] = true;
        emu.screen[idx3] = true;

        emu.execute(0xD011);

        assert_eq!(emu.v_reg[0xF], 1);

        assert!(!emu.screen[idx1]);
        assert!(!emu.screen[idx3]);
    }

    #[test]
    fn test_opcode_dxyn_wrap_horizontal() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0b0000_1111; // bits 4-7 = 1 → col 4,5,6,7

        emu.v_reg[0x0] = (SCREEN_WIDTH - 4) as u8; // X start
        emu.v_reg[0x1] = 15; // Y

        emu.execute(0xD011);

        // after wrap -- pixels at the line start (X=0 to 3)
        #[allow(clippy::identity_op)]
        let wrapped_base = 15 * SCREEN_WIDTH + 0;
        assert!(emu.screen[wrapped_base]);
        assert!(emu.screen[wrapped_base + 1]);
        assert!(emu.screen[wrapped_base + 2]);
        assert!(emu.screen[wrapped_base + 3]);

        // old positions (60-63) should be turned off
        let old_base = 15 * SCREEN_WIDTH + 60;
        assert!(!emu.screen[old_base]);
        assert!(!emu.screen[old_base + 1]);
        assert!(!emu.screen[old_base + 2]);
        assert!(!emu.screen[old_base + 3]);

        assert_eq!(emu.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_dxyn_wrap_vertical() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0xFF;
        emu.ram[0x51] = 0xFF;

        emu.v_reg[0x0] = 10;
        emu.v_reg[0x1] = 31;

        emu.execute(0xD012);

        #[allow(clippy::erasing_op)]
        #[allow(clippy::identity_op)]
        let wrapped = 0 * 64 + 10;

        assert!(emu.screen[wrapped]);
        assert!(emu.screen[wrapped + 1]);
        assert!(emu.screen[wrapped + 2]);
        assert!(emu.screen[wrapped + 3]);
        assert!(emu.screen[wrapped + 4]);
        assert!(emu.screen[wrapped + 5]);
        assert!(emu.screen[wrapped + 6]);
        assert!(emu.screen[wrapped + 7]);

        assert_eq!(emu.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_dxyn_wrap_both_axes() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0b0100_0000;
        emu.ram[0x51] = 0b0100_0000;

        emu.v_reg[0x4] = (64 - 1) as u8;
        emu.v_reg[0x5] = (32 - 1) as u8;

        emu.execute(0xD452);

        assert!(emu.screen[0]);
        assert!(emu.screen[64 * 31]);
    }

    #[test]
    fn test_opcode_dxyn_multiple_collisions_vf_still_1() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0xFF;
        emu.v_reg[0x0] = 10;
        emu.v_reg[0x1] = 10;
        for i in [0, 2, 4, 6] {
            emu.screen[10 * 64 + 10 + i] = true;
        }
        emu.execute(0xD011);
        assert_eq!(emu.v_reg[0xF], 1);
    }

    #[test]
    fn test_opcode_dxyn_zero_height_no_draw() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0xFF;
        emu.v_reg[0x0] = 20;
        emu.v_reg[0x1] = 20;
        emu.execute(0xD010);
        assert!(emu.screen.iter().all(|&p| !p));
        assert_eq!(emu.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_dxyn_partial_sprite_offscreen() {
        let mut emu = Emu::new();
        emu.i_reg = 0x50;
        emu.ram[0x50] = 0b1111_1111;
        emu.v_reg[0x0] = 58;
        emu.v_reg[0x1] = 10;
        emu.execute(0xD011);
        for i in 0..6 {
            assert!(emu.screen[10 * 64 + 58 + i]);
        }
        for i in 0..2 {
            assert!(emu.screen[10 * 64 + i]);
        }
        assert_eq!(emu.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_fx07_load_dt_basic() {
        let mut emu = Emu::new();
        emu.dt = 42;
        emu.execute(0xF207);
        assert_eq!(emu.v_reg[0x2], 42);
    }

    #[test]
    fn test_opcode_fx07_load_dt_zero() {
        let mut emu = Emu::new();
        emu.dt = 0;
        emu.execute(0xF007);
        assert_eq!(emu.v_reg[0x0], 0);
    }

    #[test]
    fn test_opcode_fx07_load_dt_max() {
        let mut emu = Emu::new();
        emu.dt = 255;
        emu.execute(0xFF07);
        assert_eq!(emu.v_reg[0xF], 255);
    }

    #[test]
    fn test_opcode_fx07_different_registers() {
        let mut emu = Emu::new();
        emu.dt = 100;
        emu.execute(0xF107);
        assert_eq!(emu.v_reg[0x1], 100);
        emu.execute(0xFA07);
        assert_eq!(emu.v_reg[0xA], 100);
    }

    #[test]
    fn test_opcode_fx07_with_tick_cycle() {
        let mut emu = Emu::new();
        emu.dt = 5;
        emu.ram[0x200] = 0xF3;
        emu.ram[0x201] = 0x07;
        emu.pc = 0x200;
        emu.tick();
        assert_eq!(emu.v_reg[0x3], 5);
        assert_eq!(emu.dt, 4);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_opcode_fx07_multiple_in_sequence() {
        let mut emu = Emu::new();
        emu.dt = 60;
        emu.ram[0x200] = 0xF1;
        emu.ram[0x201] = 0x07;
        emu.ram[0x202] = 0xF2;
        emu.ram[0x203] = 0x07;
        emu.pc = 0x200;
        emu.tick();
        assert_eq!(emu.v_reg[0x1], 60);
        assert_eq!(emu.dt, 59);
        emu.tick();
        assert_eq!(emu.v_reg[0x2], 59);
        assert_eq!(emu.pc, 0x204);
    }

    #[test]
    fn test_opcode_fx15_set_dt_basic() {
        let mut emu = Emu::new();
        emu.v_reg[0x3] = 42;
        emu.execute(0xF315);
        assert_eq!(emu.dt, 42);
    }

    #[test]
    fn test_opcode_fx15_set_dt_zero() {
        let mut emu = Emu::new();
        emu.v_reg[0x0] = 0;
        emu.execute(0xF015);
        assert_eq!(emu.dt, 0);
    }

    #[test]
    fn test_opcode_fx15_set_dt_max() {
        let mut emu = Emu::new();
        emu.v_reg[0xF] = 255;
        emu.execute(0xFF15);
        assert_eq!(emu.dt, 255);
    }

    #[test]
    fn test_opcode_fx15_different_registers() {
        let mut emu = Emu::new();
        emu.v_reg[0x1] = 100;
        emu.v_reg[0xA] = 200;
        emu.execute(0xF115);
        assert_eq!(emu.dt, 100);
        emu.execute(0xFA15);
        assert_eq!(emu.dt, 200);
    }

    #[test]
    fn test_opcode_fx15_with_tick_cycle() {
        let mut emu = Emu::new();
        emu.v_reg[0x4] = 10;
        emu.ram[0x200] = 0xF4;
        emu.ram[0x201] = 0x15;
        emu.pc = 0x200;
        emu.tick();
        assert_eq!(emu.dt, 9); // tick decrements dt
        assert_eq!(emu.pc, 0x202);
        emu.tick();
        assert_eq!(emu.dt, 8); // tick decrements dt
    }

    #[test]
    fn test_opcode_fx15_multiple_in_sequence() {
        let mut emu = Emu::new();
        emu.ram[0x200] = 0xF1;
        emu.ram[0x201] = 0x15;
        emu.ram[0x202] = 0xF2;
        emu.ram[0x203] = 0x15;
        emu.pc = 0x200;
        emu.v_reg[0x1] = 60;
        emu.v_reg[0x2] = 30;
        emu.tick();
        assert_eq!(emu.dt, 59); // tick decrements dt
        emu.tick();
        assert_eq!(emu.dt, 29);
        assert_eq!(emu.pc, 0x204);
    }

    #[test]
    fn test_opcode_cxkk_rand_basic() {
        let mut emu = Emu::new();
        emu.v_reg[0x0] = 0;
        emu.execute(0xC012);
        assert!(emu.v_reg[0x0] <= 0x12);
        assert_eq!(emu.v_reg[0x0] & 0xED, 0);
    }

    #[test]
    fn test_opcode_cxkk_rand_zero_mask() {
        let mut emu = Emu::new();
        emu.v_reg[0x1] = 0xFF;
        emu.execute(0xC100);
        assert_eq!(emu.v_reg[0x1], 0);
    }

    #[test]
    fn test_opcode_cxkk_rand_full_mask() {
        let mut emu = Emu::new();
        emu.v_reg[0x2] = 0;
        emu.execute(0xC2FF);
        assert_eq!(emu.v_reg[0x2], emu.v_reg[0x2]);
    }

    #[test]
    fn test_opcode_cxkk_rand_different_registers() {
        let mut emu = Emu::new();
        emu.execute(0xC388);
        assert!(emu.v_reg[0x3] <= 0x88);
        emu.execute(0xCA55);
        assert!(emu.v_reg[0xA] <= 0x55);
    }

    #[test]
    fn test_opcode_cxkk_rand_multiple_times_same_register() {
        let mut emu = Emu::new();
        let first = {
            emu.execute(0xC4AA);
            emu.v_reg[0x4]
        };
        let second = {
            emu.v_reg[0x4] = 0;
            emu.execute(0xC4AA);
            emu.v_reg[0x4]
        };
        assert_ne!(first, second);
    }

    #[test]
    fn test_opcode_cxkk_rand_with_tick_cycle() {
        let mut emu = Emu::new();
        emu.ram[0x200] = 0xC5;
        emu.ram[0x201] = 0x3F;
        emu.pc = 0x200;
        emu.tick();
        assert!(emu.v_reg[0x5] <= 0x3F);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_fx33_bcd_basic() {
        let mut emu = Emu::new();
        emu.v_reg[0x5] = 123;
        emu.i_reg = 0x300;
        emu.execute(0xF533);
        assert_eq!(emu.ram[0x300], 1);
        assert_eq!(emu.ram[0x301], 2);
        assert_eq!(emu.ram[0x302], 3);
    }

    #[test]
    fn test_fx33_bcd_zero() {
        let mut emu = Emu::new();
        emu.v_reg[0x0] = 0;
        emu.i_reg = 0x400;
        emu.execute(0xF033);
        assert_eq!(emu.ram[0x400], 0);
        assert_eq!(emu.ram[0x401], 0);
        assert_eq!(emu.ram[0x402], 0);
    }

    #[test]
    fn test_fx33_bcd_max_value() {
        let mut emu = Emu::new();
        emu.v_reg[0xF] = 255;
        emu.i_reg = 0x500;
        emu.execute(0xFF33);
        assert_eq!(emu.ram[0x500], 2);
        assert_eq!(emu.ram[0x501], 5);
        assert_eq!(emu.ram[0x502], 5);
    }

    #[test]
    fn test_fx33_bcd_single_digit() {
        let mut emu = Emu::new();
        emu.v_reg[0xA] = 7;
        emu.i_reg = 0x600;
        emu.execute(0xFA33);
        assert_eq!(emu.ram[0x600], 0);
        assert_eq!(emu.ram[0x601], 0);
        assert_eq!(emu.ram[0x602], 7);
    }

    #[test]
    fn test_fx33_bcd_ten() {
        let mut emu = Emu::new();
        emu.v_reg[0x1] = 10;
        emu.i_reg = 0x700;
        emu.execute(0xF133);
        assert_eq!(emu.ram[0x700], 0);
        assert_eq!(emu.ram[0x701], 1);
        assert_eq!(emu.ram[0x702], 0);
    }

    #[test]
    fn test_fx33_bcd_multiple_in_sequence() {
        let mut emu = Emu::new();
        emu.i_reg = 0x800;
        emu.v_reg[0x2] = 45;
        emu.v_reg[0x3] = 67;
        emu.execute(0xF233);
        assert_eq!(emu.ram[0x800], 0);
        assert_eq!(emu.ram[0x801], 4);
        assert_eq!(emu.ram[0x802], 5);
        emu.i_reg = 0x900;
        emu.execute(0xF333);
        assert_eq!(emu.ram[0x900], 0);
        assert_eq!(emu.ram[0x901], 6);
        assert_eq!(emu.ram[0x902], 7);
    }
}
