use chip8_core::{Chip8, FrameBuffer};
use softbuffer::{Buffer, Context, Surface};
use std::{
    default,
    num::NonZeroU32,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 256;

const WHITE_PIXEL: u32 = 0x00ffffff;
const BLACK_PIXEL: u32 = 0x00000000;
const PIXEL_SCALE: u32 = 8;

const FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / FPS);

struct RenderTarget {
    time: Instant,
}

impl Default for RenderTarget {
    fn default() -> Self {
        RenderTarget {
            time: (Instant::now()),
        }
    }
}

#[derive(Default)]
pub(super) struct App {
    render_target: RenderTarget,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    cpu: Chip8,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let path: &Path = Path::new("C:\\Users\\cnlwe\\Desktop\\repos\\gumbis8\\roms\\br8kout.ch8");
        self.cpu.load_rom(path).unwrap();

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
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        surface
            .resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )
            .unwrap();
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if self.render_target.time <= Instant::now() {
            self.render_target.time += FRAME_TIME;
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                //let size = self.window.as_ref().unwrap().inner_size();
                // if let Some(surface) = self.surface.as_mut() {
                //     surface
                //         .resize(
                //             NonZeroU32::new(size.width).unwrap(),
                //             NonZeroU32::new(size.height).unwrap(),
                //         )
                //         .unwrap();
                self.cpu.cycle();
                let mut buffer = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                chip8_draw(&mut buffer, self.cpu.get_buffer());
                buffer.present().unwrap();
                let now = Instant::now();
                if self.render_target.time <= now {
                    self.render_target.time = now + FRAME_TIME;
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                handle_input(&mut self.cpu, event);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.render_target.time,
        ));
    }
}

fn chip8_draw(winit_buffer: &mut Buffer<'_, Rc<Window>, Rc<Window>>, chip8_buffer: &FrameBuffer) {
    let b_width = winit_buffer.width().get();
    for index in 0..(chip8_core::HEIGHT * chip8_core::WIDTH) {
        let c8_x = index % 64;
        let c8_y = index / 64;

        let start_index =
            c8_x * PIXEL_SCALE as usize + c8_y * b_width as usize * PIXEL_SCALE as usize;

        let is_set = chip8_buffer.get_pixel(c8_x, c8_y);
        for i in 0..PIXEL_SCALE {
            for j in 0..PIXEL_SCALE {
                if is_set == 1 {
                    winit_buffer[start_index + (i * b_width + j) as usize] = WHITE_PIXEL;
                } else {
                    winit_buffer[start_index + (i * b_width + j) as usize] = BLACK_PIXEL;
                }
            }
        }
    }
}

fn handle_input(cpu: &mut Chip8, event: KeyEvent) {
    match event.physical_key {
        PhysicalKey::Code(KeyCode::Digit1) => cpu.input(0x1, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::Digit2) => cpu.input(0x2, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::Digit3) => cpu.input(0x3, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::Digit4) => cpu.input(0xC, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyQ) => cpu.input(0x4, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyW) => cpu.input(0x5, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyE) => cpu.input(0x6, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyR) => cpu.input(0xD, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyA) => cpu.input(0x7, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyS) => cpu.input(0x8, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyD) => cpu.input(0x9, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyF) => cpu.input(0xE, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyZ) => cpu.input(0xA, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyX) => cpu.input(0x0, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyC) => cpu.input(0xB, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::KeyV) => cpu.input(0xF, event.state.is_pressed()),
        _ => {}
    }
}
