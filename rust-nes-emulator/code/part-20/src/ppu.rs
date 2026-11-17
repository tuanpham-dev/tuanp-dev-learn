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
    vertical_mirroring: bool,
    nametables: [u8; 0x800],
    palette: [u8; 32],
    oam: [u8; 256],

    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,

    addr: u16,
    second_write: bool,
    read_buffer: u8,
    scroll_x: u8,
    scroll_y: u8,
    frame_scroll_y: usize,

    pub nmi_pending: bool,
    pub frame: Vec<u32>,
}

impl Ppu {
    pub fn new(chr: Vec<u8>, vertical_mirroring: bool) -> Ppu {
        Ppu {
            chr,
            vertical_mirroring,
            nametables: [0; 0x800],
            palette: [0; 32],
            oam: [0; 256],
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            addr: 0,
            second_write: false,
            read_buffer: 0,
            scroll_x: 0,
            scroll_y: 0,
            frame_scroll_y: 0,
            nmi_pending: false,
            frame: vec![0; WIDTH * HEIGHT],
        }
    }

    // ---- the PPU's own memory ----

    // The cartridge decides how four name tables fit into the two that exist.
    fn nametable_index(&self, addr: u16) -> usize {
        let offset = (addr as usize - 0x2000) % 0x1000;
        let table = offset / 0x400;
        let table = if self.vertical_mirroring { table % 2 } else { table / 2 };
        table * 0x400 + offset % 0x400
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
            3 => self.oam_addr = value,
            4 => {
                self.oam[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                if !self.second_write {
                    self.scroll_x = value;
                } else {
                    self.scroll_y = value;
                }
                self.second_write = !self.second_write;
            }
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
            4 => self.oam[self.oam_addr as usize],
            7 => {
                // Reads arrive one step late, through a buffer. Palette
                // reads are the exception.
                let addr = self.addr % 0x4000;
                let value = if addr < 0x3F00 {
                    let buffered = self.read_buffer;
                    self.read_buffer = self.mem_read(addr);
                    buffered
                } else {
                    self.read_buffer = self.mem_read(addr - 0x1000);
                    self.mem_read(addr)
                };
                self.step_addr();
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
        self.status &= 0x3F; // clear the vblank and sprite-0 flags
        let table_y = (self.control as usize >> 1) & 1;
        self.frame_scroll_y = table_y * 240 + self.scroll_y as usize;
    }

    // ---- drawing ----

    // Draw one horizontal line of the picture into `frame`.
    pub fn render_scanline(&mut self, y: usize) {
        // For each pixel of the line: which palette slot (0 to 31) it shows,
        // and whether the background is solid there.
        let mut slots = [0u8; WIDTH];
        let mut solid = [false; WIDTH];

        if self.mask & 0x08 != 0 {
            self.render_background(y, &mut slots, &mut solid);
        }
        if self.mask & 0x10 != 0 {
            self.render_sprites(y, &mut slots, &solid);
        }

        for x in 0..WIDTH {
            let colour = self.palette[slots[x] as usize] & 0x3F;
            self.frame[y * WIDTH + x] = COLOURS[colour as usize];
        }
    }

    fn render_background(&self, y: usize, slots: &mut [u8; WIDTH], solid: &mut [bool; WIDTH]) {
        let table_x = self.control as usize & 1;
        let scroll_x = table_x * 256 + self.scroll_x as usize;
        let tiles = if self.control & 0x10 != 0 { 256 } else { 0 };
        let first = if self.mask & 0x02 != 0 { 0 } else { 8 };

        // (wx, wy) is where this pixel falls in the 512x480 world
        // made of four name tables.
        let wy = (y + self.frame_scroll_y) % 480;
        for x in first..WIDTH {
            let wx = (x + scroll_x) % 512;
            let table = 0x2000 + ((wy / 240) * 2 + wx / 256) * 0x400;
            let col = (wx % 256) / 8;
            let row = (wy % 240) / 8;

            let tile = self.mem_read((table + row * 32 + col) as u16) as usize;
            let colour = tile_pixel(&self.chr, tiles + tile, wx % 8, wy % 8);
            if colour == 0 {
                continue;
            }

            // One attribute byte covers 4x4 tiles; two bits for each 2x2 corner.
            let attribute = self.mem_read((table + 0x3C0 + (row / 4) * 8 + col / 4) as u16);
            let corner = (row % 4 / 2) * 2 + col % 4 / 2;
            let palette = (attribute >> (corner * 2)) & 3;

            slots[x] = palette * 4 + colour;
            solid[x] = true;
        }
    }

    fn render_sprites(&mut self, y: usize, slots: &mut [u8; WIDTH], solid: &[bool; WIDTH]) {
        let height = if self.control & 0x20 != 0 { 16 } else { 8 };
        let first = if self.mask & 0x04 != 0 { 0 } else { 8 };

        // Last sprite first, so that sprite 0 ends up on top.
        for i in (0..64).rev() {
            let top = self.oam[i * 4] as usize + 1;
            let number = self.oam[i * 4 + 1] as usize;
            let attributes = self.oam[i * 4 + 2];
            let left = self.oam[i * 4 + 3] as usize;

            if y < top || y >= top + height {
                continue;
            }
            let mut row = y - top;
            if attributes & 0x80 != 0 {
                row = height - 1 - row; // flipped upside down
            }
            let tile = if height == 8 {
                let tiles = if self.control & 0x08 != 0 { 256 } else { 0 };
                tiles + number
            } else {
                (number & 1) * 256 + (number & 0xFE) + row / 8
            };

            for col in 0..8 {
                let x = left + col;
                if x >= WIDTH || x < first {
                    continue;
                }
                let tx = if attributes & 0x40 != 0 { 7 - col } else { col }; // mirrored
                let colour = tile_pixel(&self.chr, tile, tx, row % 8);
                if colour == 0 {
                    continue;
                }
                if i == 0 && solid[x] && x != 255 {
                    self.status |= 0x40; // sprite 0 touched the background
                }
                if attributes & 0x20 != 0 && solid[x] {
                    continue; // this sprite goes behind the background
                }
                slots[x] = 16 + (attributes & 3) * 4 + colour;
            }
        }
    }
}
