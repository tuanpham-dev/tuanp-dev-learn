// The colours a NES can show, as 0xRRGGBB. A palette entry is a number
// from 0 to 63 that picks one of these.
const COLOURS: [u32; 64] = [
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00,
    0x333500, 0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000,
    0xADADAD, 0x155FD9, 0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00,
    0x6B6D00, 0x388700, 0x0C9300, 0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000,
    0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF, 0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22,
    0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE, 0x4F4F4F, 0x000000, 0x000000,
    0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA, 0xFECCC5, 0xF7D8A5,
    0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000, 0x000000,
];

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

// The colour (0 to 3) of one pixel of one tile.
fn tile_pixel(chr: &[u8], tile: usize, x: usize, y: usize) -> u8 {
    let lo = chr[tile * 16 + y];
    let hi = chr[tile * 16 + y + 8];
    let shift = 7 - x;
    let lo_bit = (lo >> shift) & 1;
    let hi_bit = (hi >> shift) & 1;
    hi_bit * 2 + lo_bit
}

pub struct Ppu {
    chr: Vec<u8>,
    nametables: [u8; 0x800],
    palette: [u8; 32],

    control: u8,
    mask: u8,
    status: u8,

    addr: u16,
    second_write: bool,

    pub nmi_pending: bool,
    pub frame: Vec<u32>,
}

impl Ppu {
    pub fn new(chr: Vec<u8>) -> Ppu {
        Ppu {
            chr,
            nametables: [0; 0x800],
            palette: [0; 32],
            control: 0,
            mask: 0,
            status: 0,
            addr: 0,
            second_write: false,
            nmi_pending: false,
            frame: vec![0; WIDTH * HEIGHT],
        }
    }

    // ---- the PPU's own memory ----

    // Where a name-table address lands in our two tables' worth of memory.
    fn nametable_index(&self, addr: u16) -> usize {
        (addr as usize - 0x2000) % 0x800
    }

    // Colour 0 of every sprite palette is the same slot as in the
    // matching background palette.
    fn palette_index(addr: u16) -> usize {
        let index = addr as usize % 32;
        if index >= 16 && index % 4 == 0 { index - 16 } else { index }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        let addr = addr % 0x4000;
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize],
            0x2000..=0x3EFF => self.nametables[self.nametable_index(addr)],
            _ => self.palette[Ppu::palette_index(addr)],
        }
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        let addr = addr % 0x4000;
        match addr {
            0x0000..=0x1FFF => {} // the tiles are ROM: writes do nothing
            0x2000..=0x3EFF => self.nametables[self.nametable_index(addr)] = value,
            _ => self.palette[Ppu::palette_index(addr)] = value,
        }
    }

    // ---- the eight registers the processor can see ----

    pub fn write_register(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                let was_on = self.control & 0x80 != 0;
                self.control = value;
                // Switching the interrupt on in the middle of vblank fires it at once.
                if !was_on && value & 0x80 != 0 && self.status & 0x80 != 0 {
                    self.nmi_pending = true;
                }
            }
            1 => self.mask = value,
            6 => {
                if !self.second_write {
                    self.addr = (self.addr & 0x00FF) | ((value as u16) << 8);
                } else {
                    self.addr = (self.addr & 0xFF00) | value as u16;
                }
                self.second_write = !self.second_write;
            }
            7 => {
                self.mem_write(self.addr, value);
                self.step_addr();
            }
            _ => {}
        }
    }

    pub fn read_register(&mut self, register: u16) -> u8 {
        match register {
            2 => {
                let value = self.status;
                self.status &= 0x7F; // reading clears the vblank flag
                self.second_write = false;
                value
            }
            _ => 0,
        }
    }

    fn step_addr(&mut self) {
        let step = if self.control & 0x04 != 0 { 32 } else { 1 };
        self.addr = self.addr.wrapping_add(step);
    }

    // ---- timing ----

    pub fn start_vblank(&mut self) {
        self.status |= 0x80;
        if self.control & 0x80 != 0 {
            self.nmi_pending = true;
        }
    }

    pub fn end_vblank(&mut self) {
        self.status &= 0x7F;
    }

    // ---- drawing ----

    // Draw one horizontal line of the picture into `frame`.
    pub fn render_scanline(&mut self, y: usize) {
        // For each pixel of the line: which palette slot (0 to 15) it shows.
        let mut slots = [0u8; WIDTH];

        if self.mask & 0x08 != 0 {
            self.render_background(y, &mut slots);
        }

        for x in 0..WIDTH {
            let colour = self.palette[slots[x] as usize] & 0x3F;
            self.frame[y * WIDTH + x] = COLOURS[colour as usize];
        }
    }

    fn render_background(&self, y: usize, slots: &mut [u8; WIDTH]) {
        let row = y / 8;
        for x in 0..WIDTH {
            let col = x / 8;

            let tile = self.mem_read((0x2000 + row * 32 + col) as u16) as usize;
            let colour = tile_pixel(&self.chr, tile, x % 8, y % 8);
            if colour == 0 {
                continue;
            }

            // One attribute byte covers 4x4 tiles; two bits for each 2x2 corner.
            let attribute = self.mem_read((0x23C0 + (row / 4) * 8 + col / 4) as u16);
            let corner = (row % 4 / 2) * 2 + col % 4 / 2;
            let palette = (attribute >> (corner * 2)) & 3;

            slots[x] = palette * 4 + colour;
        }
    }
}
