// One NES controller. The eight buttons are the eight bits of `buttons`:
// A, B, Select, Start, Up, Down, Left, Right, from bit 0 upwards.
pub struct Controller {
    pub buttons: u8,
    shift: u8,
    strobe: bool,
}

pub const BUTTON_A: u8 = 0x01;
pub const BUTTON_B: u8 = 0x02;
pub const BUTTON_SELECT: u8 = 0x04;
pub const BUTTON_START: u8 = 0x08;
pub const BUTTON_UP: u8 = 0x10;
pub const BUTTON_DOWN: u8 = 0x20;
pub const BUTTON_LEFT: u8 = 0x40;
pub const BUTTON_RIGHT: u8 = 0x80;

impl Controller {
    pub fn new() -> Controller {
        Controller { buttons: 0, shift: 0, strobe: false }
    }

    // A write to $4016. While bit 0 is on, the controller keeps taking
    // a fresh snapshot of the buttons.
    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 != 0;
        if self.strobe {
            self.shift = self.buttons;
        }
    }

    // A read from $4016: hand over one button, then line up the next.
    pub fn read(&mut self) -> u8 {
        if self.strobe {
            return self.buttons & 1;
        }
        let bit = self.shift & 1;
        self.shift = (self.shift >> 1) | 0x80;
        bit
    }
}
