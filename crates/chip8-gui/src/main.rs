use winit::event_loop::EventLoop;
mod app;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = app::App::default();
    event_loop.run_app(&mut app).unwrap();
}
