mod error;
mod gfx;
mod hresult;
mod renderer;

use anyhow::Result;
use dotenv::dotenv;
use gfx::{Backend, BackendD3D12, Instance};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};

#[derive(Debug)]
pub enum UiUserEvent {
    Test,
}

fn main() -> Result<()> {
    run_with::<BackendD3D12>()
}

fn run_with<B>() -> Result<()>
where
    B: Backend,
{
    dotenv().ok();
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let window_width = 1280f64;
    let window_height = 720f64;
    let event_loop: EventLoop<UiUserEvent> = EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            window_width,
            window_height,
        )))
        .with_title("D3D12 Window")
        .build(&event_loop)
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    let instance = B::Instance::new(Default::default())?;
    let adapters = instance.enumerate_adapters()?;

    log::info!("available adapters:");
    for info in adapters.iter() {
        log::info!("{:?}", info.description);
    }

    runtime.block_on(async move {
        tokio::spawn(async move {
            loop {
                event_loop_proxy
                    .send_event(UiUserEvent::Test)
                    .expect("event loop is running");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    });

    event_loop.run(move |event, _, control_flow| {
        _ = runtime;
        _ = instance;

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::UserEvent(_) => {
                println!("user event");
            }
            _ => (),
        }
    });
}
