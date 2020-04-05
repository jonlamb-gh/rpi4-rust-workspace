#![no_std]

extern crate bcm2711_hal as hal;
pub extern crate embedded_graphics;

use crate::hal::cache;
use crate::hal::dma;
use crate::hal::mailbox::{AllocFramebufferRepr, PixelOrder};
pub use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::*;
use embedded_graphics::DrawTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// One of the PMem parameters is too small
    InsufficientPMem,
    /// Framebuffer physical address not in frontbuffer
    MissingFramebuffer,
    /// Framebuffer config not supported
    FramebufferConfig,
    /// Provided a 'lite' DMA engine, but needs a standard
    DmaEngine,
    /// DMA transfer failed
    DmaTransfer,
}

/// Uses a single DMA control block
pub const DCB_MEM_MIN_SIZE: usize = dma::CONTROL_BLOCK_SIZE;

/// Reserve 16 u32 words (512 bytes) for the scratchpad/fill-buffer
pub const SCRATCHPAD_MEM_MIN_SIZE: usize = 16 * 4;

/// Fill words to be used by the DMA engine when doing color fills, up to
/// 128 bit writes are supported
const NUM_FILL_WORDS: usize = 4;

// TODO - generic array to move WxH into the type?
pub struct Display<'a> {
    fb_info: AllocFramebufferRepr,
    /// DMA channel, must support 2D transfers
    dma: dma::Channel,
    /// DMA-able control block memory
    dcb_mem: &'a mut [dma::ControlBlock],
    /// DMA-able scratchpad memory
    scratchpad_mem: &'a mut [u32],
    /// DMA-able general memory for the backbuffer, must be greater than
    /// or equal to the frontbuffer in size
    backbuffer_mem: &'a mut [u32],
    /// DMA-able VideoCore GPU framebuffer memory
    frontbuffer_mem: &'a mut [u32],
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TransferOp {
    /// Fill the backbuffer with a value
    FillBack,
    /// Fill the frontbuffer with a value
    FillFront,
    /// Copy the back buffer to the frontbuffer (typically GPU memory)
    CopyBackToFront,
}

impl<'a> Display<'a> {
    pub fn new(
        fb_info: AllocFramebufferRepr,
        dma: dma::Channel,
        dcb_mem: &'a mut [dma::ControlBlock],
        scratchpad_mem: &'a mut [u32],
        backbuffer_mem: &'a mut [u32],
        frontbuffer_mem: &'a mut [u32],
    ) -> Result<Self, Error> {
        if dma.is_lite() == true {
            return Err(Error::DmaEngine);
        }

        if (fb_info.depth != 32) || (fb_info.pitch() == 0) {
            return Err(Error::FramebufferConfig);
        }

        // TODO - can lift this contiguous restriction
        if (fb_info.pitch() / 4) != fb_info.phy_width {
            return Err(Error::FramebufferConfig);
        }

        // TODO - probably can lift this and use virt
        if (fb_info.phy_width != fb_info.virt_width) || (fb_info.phy_height != fb_info.virt_height)
        {
            return Err(Error::FramebufferConfig);
        }

        if (fb_info.x_offset != 0) || (fb_info.y_offset != 0) {
            return Err(Error::FramebufferConfig);
        }

        // Make sure the frontbuffer actually holds the framebuffer memory
        if frontbuffer_mem.as_ptr() as u32 != fb_info.alloc_buffer_address() {
            return Err(Error::MissingFramebuffer);
        }

        if fb_info.alloc_buffer_size() != (frontbuffer_mem.len() * 4) {
            return Err(Error::InsufficientPMem);
        }

        // 4K page aligned
        if fb_info.buffer_align != 4096 {
            return Err(Error::FramebufferConfig);
        }

        // Sanity check real size, 32 bit depth / 8 = 4 bytes per pixel
        if fb_info.phy_width * fb_info.phy_height * (fb_info.depth / 8)
            != fb_info.alloc_buffer_size()
        {
            return Err(Error::FramebufferConfig);
        }

        if dcb_mem.len() < 1 {
            return Err(Error::InsufficientPMem);
        }

        if (scratchpad_mem.len() * 4) < SCRATCHPAD_MEM_MIN_SIZE {
            return Err(Error::InsufficientPMem);
        }

        // Backbuffer needs to be at least the size of frontbuffer
        if backbuffer_mem.len() < frontbuffer_mem.len() {
            return Err(Error::InsufficientPMem);
        }

        Ok(Display {
            fb_info,
            dma,
            dcb_mem,
            scratchpad_mem,
            backbuffer_mem,
            frontbuffer_mem,
        })
    }

    pub fn width(&self) -> usize {
        self.fb_info.virt_width
    }

    pub fn height(&self) -> usize {
        self.fb_info.virt_height
    }

    pub fn pitch(&self) -> usize {
        self.fb_info.pitch()
    }

    pub fn pixel_order(&self) -> PixelOrder {
        self.fb_info.pixel_order
    }

    #[inline]
    fn pixel_word<C: RgbColor>(pixel_order: PixelOrder, color: &C) -> u32 {
        // TODO
        match pixel_order {
            PixelOrder::RGB => {
                0xFF_00_00_00
                    | color.r() as u32
                    | (color.g() as u32) << 8
                    | (color.b() as u32) << 16
            }
            PixelOrder::BGR => {
                0xFF_00_00_00
                    | color.b() as u32
                    | (color.g() as u32) << 8
                    | (color.r() as u32) << 16
            }
            _ => unimplemented!("Unsupported pixel color {:?}", pixel_order),
        }
    }

    /// Sets a pixel in the backbuffer
    #[inline]
    pub fn set_pixel<C: RgbColor>(&mut self, x: usize, y: usize, color: &C) {
        let color_word = Self::pixel_word(self.pixel_order(), color);

        // The frontbuffer, may not be contiguous so must use pitch (pitch >= bpp*width)
        // let offset = (y * (self.pitch / 4)) + x;
        // unsafe { ptr::write(self.framebuffer.as_mut_ptr()
        // .offset(offset as _), color_word) };

        // The backbuffer is contiguous
        let offset = (y * self.width()) + x;
        self.backbuffer_mem[offset as usize] = color_word;
    }

    /// Sets a pixel in the backbuffer
    #[inline]
    pub fn set_pixel_at<C: RgbColor>(&mut self, offset: usize, color: &C) {
        let color_word = Self::pixel_word(self.pixel_order(), color);

        // The frontbuffer, may not be contiguous so must use pitch (pitch >= bpp*width)
        // let offset = (y * (self.pitch / 4)) + x;
        // unsafe { ptr::write(self.framebuffer.as_mut_ptr()
        // .offset(offset as _), color_word) };

        // The backbuffer is contiguous
        self.backbuffer_mem[offset as usize] = color_word;
    }

    /// Fills the backbuffer with a color pixel by pixel
    pub fn fill_pixels<C: RgbColor>(&mut self, color: &C) {
        let color_word = Self::pixel_word(self.pixel_order(), color);

        // Since the backbuffer is contiguous, we can use memset/alike
        for word in self.backbuffer_mem.iter_mut() {
            *word = color_word;
        }
    }

    /// Fills the backbuffer with a color using a DMA transfer
    pub fn fill_color<C: RgbColor>(&mut self, color: &C) -> Result<(), Error> {
        self.set_scratchpad_src_fill_words(color);

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack)?;
        //
        // Fill it manually for now
        self.fill_pixels(color);

        Ok(())
    }

    /// Clears the backbuffer and the frontbuffer
    pub fn clear_screen(&mut self) -> Result<(), Error> {
        self.set_scratchpad_src_fill_words::<Rgb888>(&RgbColor::BLACK);

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack)?;
        //
        // Clear it manually for now
        self.fill_pixels::<Rgb888>(&RgbColor::BLACK);

        self.swap_buffers()
    }

    /// Clears the backbuffer
    pub fn clear_buffer(&mut self) -> Result<(), Error> {
        self.set_scratchpad_src_fill_words::<Rgb888>(&RgbColor::BLACK);

        // TODO - DMA to backbuffer currently broken
        //self.dma_transfer(TransferOp::FillBack)?;
        //
        // Clear it manually for now
        self.fill_pixels::<Rgb888>(&RgbColor::BLACK);

        Ok(())
    }

    /// Swap/copy the backbuffer to the frontbuffer/framebuffer
    pub fn swap_buffers(&mut self) -> Result<(), Error> {
        self.dma_transfer(TransferOp::CopyBackToFront)
    }

    /// Constructs the DMA source fill words in the internal scratchpad buffer
    fn set_scratchpad_src_fill_words<C: RgbColor>(&mut self, color: &C) {
        let color_word = Self::pixel_word(self.pixel_order(), color);

        for w in &mut self.scratchpad_mem[..NUM_FILL_WORDS] {
            *w = color_word;
        }
    }

    fn dma_transfer(&mut self, op: TransferOp) -> Result<(), Error> {
        // Stride, in bytes, is a signed inc/dec applied after end of each row
        let bbp: usize = 4;
        let frontbuffer_stride = (self.pitch() - (self.width() * bbp)) as u32;
        let backbuffer_stride = 0;

        // Both the backbuffer and the scratchpad words are contiguous
        let src_stride = 0;

        // TODO - take out explicit cache ops once DMA impl is refactored

        let (src_inc, src_paddr, src_size, dest_paddr, dest_stride) = match op {
            TransferOp::FillBack => {
                // todo!("Fixes and dest cache ops");

                // Filling the backbuffer with the contents of the scratchpad words
                (
                    false,
                    self.scratchpad_mem.as_ptr() as u32,
                    self.scratchpad_mem.len() * 4,
                    self.backbuffer_mem.as_ptr() as u32,
                    backbuffer_stride,
                )
            }
            TransferOp::FillFront => {
                // todo!("Fixes and dest cache ops");

                // Filling the frontbuffer with the contents of the scratchpad words
                (
                    false,
                    self.scratchpad_mem.as_ptr() as u32,
                    self.scratchpad_mem.len() * 4,
                    self.frontbuffer_mem.as_ptr() as u32,
                    frontbuffer_stride,
                )
            }
            TransferOp::CopyBackToFront => {
                // Copy the backbuffer to the frontbuffer
                (
                    true,
                    self.backbuffer_mem.as_ptr() as u32,
                    self.backbuffer_mem.len() * 4,
                    self.frontbuffer_mem.as_ptr() as u32,
                    frontbuffer_stride,
                )
            }
        };

        unsafe {
            cache::clean_and_invalidate_data_cache_range(src_paddr as _, src_size);
        }

        // This is not really obvious from the DMA documentation,
        // but the top 16 bits must be programmmed to "height -1"
        // and not "height" in 2D mode.
        let transfer_length = dma::TransferLength::Mode2D(
            // Transfer length in bytes of a row
            (bbp * self.width()) as _,
            // How many x-length transfers are performed
            (self.height() - 1) as _,
        );

        // Initialize a DMA control block for the transfer
        let dcb = &mut self.dcb_mem[0];
        dcb.init();

        // Configure the DCB
        dcb.set_length(transfer_length);
        dcb.set_src(src_paddr);
        dcb.set_src_width(dma::TransferWidth::Bits128);
        dcb.stride.set_src_stride(src_stride);
        dcb.info.set_src_inc(src_inc);

        dcb.set_dest(dest_paddr);
        dcb.set_dest_width(dma::TransferWidth::Bits128);
        dcb.stride.set_dest_stride(dest_stride);
        dcb.info.set_dest_inc(true);

        dcb.info.set_wait_resp(true);
        dcb.info.set_burst_len(4);

        // TODO - hack until I redo the DMA impl
        // src/dst refs are not used
        let unused_src_buffer: [u32; 0] = [];
        let mut unused_dest_buffer: [u32; 0] = [];

        let txfr_res = dma::TransferResources {
            src_cached: false,
            dest_cached: false,
            dcb: &dcb,
            src_buffer: &unused_src_buffer,
            dest_buffer: &mut unused_dest_buffer,
        };

        // Wait for DMA to be ready, then do the transfer
        while self.dma.is_busy() == true {
            hal::cortex_a::asm::nop();
        }
        self.dma.start(&txfr_res);
        self.dma.wait();

        if self.dma.errors() {
            Err(Error::DmaTransfer)
        } else {
            Ok(())
        }
    }
}

impl<'a, C> DrawTarget<C> for Display<'a>
where
    C: RgbColor,
{
    type Error = core::convert::Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<C>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        if (coord[0] as usize) < self.width() && (coord[1] as usize) < self.height() {
            self.set_pixel(coord[0] as _, coord[1] as _, &color);
        }

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(self.width() as _, self.height() as _)
    }
}
