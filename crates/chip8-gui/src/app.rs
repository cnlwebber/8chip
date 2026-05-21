use chip8_core::{HEIGHT, WIDTH};
use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, rc::Rc};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasRawWindowHandle},
    window::{Window, WindowId},
};

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 256;

const WHITE_PIXEL: u32 = 0x00ffffff;
const BLACK_PIXEL: u32 = 0x00000000;
const PIXEL_SCALE: u32 = 8;

#[derive(Default)]
pub(super) struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Chip 8")
                        .with_inner_size(size),
                )
                .unwrap(),
        );
        let context = Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                //draw
                let size = self.window.as_ref().unwrap().inner_size();
                if let Some(surface) = self.surface.as_mut() {
                    surface
                        .resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        )
                        .unwrap();

                    // Buffer consists of one u32 for each pixel in the area to draw, so 512 * 256 u32's
                    // Pixel format (u32): 00000000RRRRRRRRGGGGGGGGBBBBBBBB
                    // White background    00000000111111111111111111111111 (0x00ffffff)
                    let mut buffer = surface.buffer_mut().unwrap();
                    let b_width = buffer.width().get();
                    let b_height = buffer.height().get();
                    for index in 0..(chip8_core::HEIGHT * chip8_core::WIDTH) {
                        let c8_x = index % 64;
                        let c8_y = index / 64;

                        let start_index = c8_x * PIXEL_SCALE as usize
                            + c8_y * b_width as usize * PIXEL_SCALE as usize;

                        for i in 0..PIXEL_SCALE {
                            for j in 0..PIXEL_SCALE {
                                if (c8_x + c8_y) % 2 == 1 {
                                    buffer[start_index + (i * b_width + j) as usize] = WHITE_PIXEL;
                                }
                            }
                        }
                    }

                    buffer.present().unwrap();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
