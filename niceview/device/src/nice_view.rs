use core::mem::{self, transmute};

use esp_hal::{
    dma::{aligned::DmaAlignedMut, DmaRxBuf, DmaTxBuf},
    gpio::Output,
    spi::master::SpiDma,
    Async,
};
use niceview_lib::{color::Color, KeyboardDisplay};

/// Display width in pixels.
const WIDTH: usize = 160;
/// Display height in pixels.
const HEIGHT: usize = 68;

// NiceView SPI command.
#[derive(Clone, Copy)]
#[repr(transparent)]
struct DisplayCommand(pub u8);
impl DisplayCommand {
    pub const WRITE: Self = DisplayCommand(0x01); // 0x80 in LSB format
    #[allow(dead_code)]
    // TODO: what is this?
    pub const VCOM: Self = DisplayCommand(0x02); // 0x40 in LSB format
    pub const CLEAR: Self = DisplayCommand(0x04); // 0x20 in LSB format
}

/// Number of octet lines in the display.
const LINE_LEN: usize = WIDTH / 8;

#[repr(C, packed)]
struct DrawCmd {
    pub command: DisplayCommand,
    pub lines: [DrawCmdLine; HEIGHT],
    pub _trailing_zero: u8,
}

#[repr(C, packed)]
struct DrawCmdLine {
    pub line_number: u8,
    pub pixel_octets: [u8; LINE_LEN],
    pub _trailing_zero: u8,
}

pub struct NiceView {
    spi: SpiDma<'static, Async>,
    cs: Output<'static>,

    /// Buffer holding the draw command, including pixel data.
    draw_command: DrawCmd,
}

impl KeyboardDisplay for NiceView {
    async fn clear_display(&mut self) {
        // TODO: what does the VCOM bit do??
        //self.write(&[self.vcom | DisplayCommand::CLEAR, 0x00]).await;
        self.write(&[DisplayCommand::CLEAR.0, 0x00]).await;
        self.toggle_vcom();
    }

    async fn flush(&mut self) {
        self.draw_command.command = DisplayCommand::WRITE;

        let cmd: &[u8; size_of::<DrawCmd>()] = unsafe { transmute(&self.draw_command) };
        self.write(cmd).await;
    }

    fn fill_white(&mut self) {
        for line in &mut self.draw_command.lines {
            for octet in &mut line.pixel_octets {
                *octet = 0xff;
            }
        }
    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        let line_octet = x >> 3;

        if line_octet >= LINE_LEN || y >= HEIGHT {
            return;
        }

        let octet_bit = x & 0b000_0111;
        let octet_mask = 1 << octet_bit;

        let line = &mut self.draw_command.lines[y];
        let octet = &mut line.pixel_octets[line_octet];

        match color {
            // Set the bit to 0 (black)
            Color::Black => *octet &= !octet_mask,

            // Set the bit to 1 (white)
            Color::White => *octet |= octet_mask,
        }
    }

    async fn write(&mut self, data: &[u8]) {
        self.cs.set_high();
        self.spi.write_async(data).await.unwrap();
        self.cs.set_low();
    }
}

impl NiceView {
    pub fn new(spi: SpiDma<'static, Async>, cs: Output<'static>) -> Self {
        let (rx_buf, rx_descriptors, tx_buf, tx_descriptors) = esp_hal::dma_buffers!(256, 256);

        let tx_buf = DmaTxBuf::new(
            DmaAlignedMut::new(tx_descriptors).unwrap(),
            DmaAlignedMut::new(tx_buf).unwrap(),
        )
        .unwrap();
        let rx_buf = DmaRxBuf::new(
            DmaAlignedMut::new(rx_descriptors).unwrap(),
            DmaAlignedMut::new(rx_buf).unwrap(),
        )
        .unwrap();
        let spi = spi.with_buffers(rx_buf, tx_buf);

        NiceView {
            spi,
            cs,
            draw_command: DrawCmd::new(),
        }
    }

    fn toggle_vcom(&mut self) {
        /*
        // ported directly adafruit driver
        // TODO: wtf is this shit?
        if self.vcom == 0 {
            self.vcom = SHARPMEM_BIT_VCOM
        } else {
            self.vcom = 0;
        }
        */
    }
}

impl DrawCmd {
    pub fn new() -> Self {
        // SAFETY: DrawCmd is repr(C, packed), and only contains types with valid zero-bitpatterns.
        let mut this: Self = unsafe { mem::zeroed() };

        for (n, line) in this.lines.iter_mut().enumerate() {
            line.line_number = n as u8;
        }

        this
    }
}
