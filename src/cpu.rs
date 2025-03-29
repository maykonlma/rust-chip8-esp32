use esp_idf_svc::hal::gpio::PinDriver;
use std::thread;
use std::time::Duration;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const FONTSET_SIZE: usize = 80;
pub const NUM_KEYS: usize = 16;
pub const NUM_REGS: usize = 16;
pub const RAM_SIZE: usize = 4096;
pub const STACK_SIZE: usize = 16;
pub const START_ADDR: u16 = 0x200;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    pub dt: u8,
    pub i_reg: u16,
    pub keys: [bool; NUM_KEYS],
    pub pc: u16,
    pub ram: Box<[u8; RAM_SIZE]>, // Alocado no heap
    pub screen: Box<[bool; SCREEN_WIDTH * SCREEN_HEIGHT]>, // Alocado no heap
    pub sp: u16,
    pub st: u8,
    pub stack: [u16; STACK_SIZE],
    pub v_reg: [u8; NUM_REGS],
}

impl Cpu { 
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: Box::new([0; RAM_SIZE]),
            screen: Box::new([false; SCREEN_WIDTH * SCREEN_HEIGHT]),
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

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => self.opcode_00e0(),
            (0, 0, 0xE, 0xE) => self.opcode_00ee(),
            (1, _, _, _) => self.opcode_1nnn(op),
            (2, _, _, _) => self.opcode_2nnn(op),
            (3, _, _, _) => self.opcode_3xnn(op, digit2),
            (4, _, _, _) => self.opcode_4xnn(op, digit2),
            (5, _, _, _) => self.opcode_5xy0(digit2, digit3),
            (6, _, _, _) => self.opcode_6xnn(op, digit2),
            (7, _, _, _) => self.opcode_7xnn(op, digit2),
            (8, _, _, 0) => self.opcode_8xy0(digit2, digit3),
            (8, _, _, 1) => self.opcode_8xy1(digit2, digit3),
            (8, _, _, 2) => self.opcode_8xy2(digit2, digit3),
            (8, _, _, 3) => self.opcode_8xy3(digit2, digit3),
            (8, _, _, 4) => self.opcode_8xy4(digit2, digit3),
            (8, _, _, 5) => self.opcode_8xy5(digit2, digit3),
            (8, _, _, 6) => self.opcode_8xy6(digit2),
            (8, _, _, 7) => self.opcode_8xy7(digit2, digit3),
            (8, _, _, 0xE) => self.opcode_8xye(digit2),
            (9, _, _, 0) => self.opcode_9xy0(digit2, digit3),
            (0xA, _, _, _) => self.opcode_annn(op),
            (0xB, _, _, _) => self.opcode_bnnn(op),
            (0xC, _, _, _) => self.opcode_cxnn(op, digit2),
            (0xD, _, _, _) => self.opcode_dxyn(digit2, digit3, digit4),
            (0xE, _, 9, 0xE) => self.opcode_ex9e(digit2),
            (0xE, _, 0xA, 1) => self.opcode_exa1(digit2),
            (0xF, _, 0, 7) => self.opcode_fx07(digit2),
            (0xF, _, 0, 0xA) => self.opcode_fx0a(digit2),
            (0xF, _, 1, 5) => self.opcode_fx15(digit2),
            (0xF, _, 1, 8) => self.opcode_fx18(digit2),
            (0xF, _, 1, 0xE) => self.opcode_fx1e(digit2),
            (0xF, _, 2, 9) => self.opcode_fx29(digit2),
            (0xF, _, 3, 3) => self.opcode_fx33(digit2),
            (0xF, _, 5, 5) => self.opcode_fx55(digit2),
            (0xF, _, 6, 5) => self.opcode_fx65(digit2),
            (_, _, _, _) => unimplemented!("Opcode não implementado: {:#04x}.", op),
        }
    }

    fn play_beep(buzzer_pin: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio4, esp_idf_svc::hal::gpio::Output>) {
        // Liga o buzzer (seta o pino como HIGH)
        buzzer_pin.set_high().unwrap();
    
        // Mantém o bip por 100 ms
        thread::sleep(Duration::from_millis(100));
    
        // Desliga o buzzer (seta o pino como LOW)
        buzzer_pin.set_low().unwrap();
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = Box::new([0; RAM_SIZE]);
        self.screen = Box::new([false; SCREEN_WIDTH * SCREEN_HEIGHT]);
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    pub fn tick_timers_beep(&mut self, buzzer_pin: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Gpio4, esp_idf_svc::hal::gpio::Output>) {
        if self.st > 0 {
            if self.st == 1 {
                Cpu::play_beep(buzzer_pin);
            }
            self.st -= 1;
        }
    }
}

impl Cpu {
    fn opcode_fx65(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.v_reg[idx] = self.ram[i + idx];
        }
    }

    fn opcode_fx55(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.ram[i + idx] = self.v_reg[idx];
        }
    }
    
    fn opcode_fx33(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x] as f32;
         
        let hundreds = (vx / 100.0).floor() as u8;
        
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        
        let ones = (vx % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }
    
    fn opcode_fx29(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let c = self.v_reg[x] as u16;
        self.i_reg = c * 5;
    }
    
    fn opcode_fx1e(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x] as u16;
        self.i_reg = self.i_reg.wrapping_add(vx);
    }
    
    fn opcode_fx18(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.st = self.v_reg[x];
    }

    fn opcode_fx15(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.dt = self.v_reg[x];
    }

    fn opcode_fx0a(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.pc -= 2;
        }
    }

    fn opcode_fx07(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.v_reg[x] = self.dt;
    }
    
    fn opcode_exa1(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if !key {
            self.pc += 2;
        }
    }
    
    fn opcode_ex9e(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if key {
            self.pc += 2;
        }
    }
    
    fn opcode_dxyn(&mut self, digit2: u16, digit3: u16, digit4: u16) {
        let x_coord = self.v_reg[digit2 as usize] as u16;
        let y_coord = self.v_reg[digit3 as usize] as u16;
        let num_rows = digit4;

        let mut flipped = false;
        for y_line in 0..num_rows {
            let addr = self.i_reg + y_line;
            let pixels = self.ram[addr as usize];

            for x_line in 0..8 {
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                    
                    let idx = x + SCREEN_WIDTH * y;
                    
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }
        
        if flipped {
            self.v_reg[0xF] = 1;
        } else {
            self.v_reg[0xF] = 0;
        }
    }

    fn opcode_cxnn(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        let rng = unsafe { esp_idf_svc::sys::rand() as u8 };
        self.v_reg[x] = rng & nn;
    }
    
    fn opcode_bnnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = (self.v_reg[0] as u16) + nnn;
    }
    
    fn opcode_annn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.i_reg = nnn;
    }
    
    fn opcode_9xy0(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }
    
    fn opcode_8xye(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let msb = (self.v_reg[x] >> 7) & 1;
        self.v_reg[x] <<= 1;
        self.v_reg[0xF] = msb;
    }
    
    fn opcode_8xy7(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }
    
    fn opcode_8xy6(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let lsb = self.v_reg[x] & 1;
        self.v_reg[x] >>= 1;
        self.v_reg[0xF] = lsb;
    }
    
    fn opcode_8xy5(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }
    
    fn opcode_8xy4(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        let new_vf = if carry { 1 } else { 0 };

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;
    }
    
    fn opcode_8xy3(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] ^= self.v_reg[y];
    }
    
    fn opcode_8xy2(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] &= self.v_reg[y];
    }
    
    fn opcode_8xy1(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] |= self.v_reg[y];
    }
    
    fn opcode_8xy0(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] = self.v_reg[y];
    }
    
    fn opcode_7xnn(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
    }
    
    fn opcode_6xnn(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = nn;
    }
    
    fn opcode_5xy0(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }
    
    fn opcode_4xnn(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x] != nn {
            self.pc += 2;
        }
    }
    
    fn opcode_3xnn(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x] == nn {
            self.pc += 2;
        }
    }
    
    fn opcode_2nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.push(self.pc);
        self.pc = nnn;
    }
    
    fn opcode_1nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = nnn;
    }

    fn opcode_00ee(&mut self) {
        let ret_addr = self.pop();
        self.pc = ret_addr;
    }
    
    fn opcode_00e0(&mut self) {
        self.screen = Box::new([false; SCREEN_WIDTH * SCREEN_HEIGHT]);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
}