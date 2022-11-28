mod error;
mod gfx;
mod hresult;
mod renderer;

use std::{path::Path, sync::Arc};

use anyhow::Result;
use dotenv::dotenv;
use gfx::{Backend, BackendD3D12};
use image::{Rgb, RgbaImage};
use tokio::sync::mpsc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

#[derive(Debug)]
pub struct ImageLibrary {
    pub dragon_texture_ao_specular_reflection: RgbaImage,
    pub dragon_texture_color: RgbaImage,
    pub dragon_texture_normal: RgbaImage,
    pub plane_texture_color: RgbaImage,
    pub plane_texture_depthmap: RgbaImage,
    pub plane_texture_normal: RgbaImage,
    pub suzanne_texture_ao_specular_reflection: RgbaImage,
    pub suzanne_texture_color: RgbaImage,
    pub suzanne_texture_normal: RgbaImage,
}

#[derive(Debug)]
pub enum ControlToUiEvent {
    Test,
}

#[derive(Debug)]
pub enum UiToControlEvent {
    Started,
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
    let event_loop: EventLoop<ControlToUiEvent> = EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            window_width,
            window_height,
        )))
        .with_title("Demo")
        .build(&event_loop)
        .unwrap();
    let (control_tx, mut control_rx) = mpsc::unbounded_channel();
    control_tx
        .send(UiToControlEvent::Started)
        .expect("failed to send message");

    let event_loop_proxy = event_loop.create_proxy();

    runtime.block_on(async move {
        tokio::spawn(async move {
            while let Some(evt) = control_rx.recv().await {
                if let Err(err) = handle_control_event(evt).await {
                    log::error!("handle control event error: {}", err);
                }
            }
        });
    });

    event_loop.run(move |event, _, control_flow| {
        if let Err(err) = handle_ui_event(&window, event, control_flow) {
            log::error!("handle ui event error: {}", err);
        }
    });
}

async fn handle_control_event(event: UiToControlEvent) -> Result<()> {
    match event {
        UiToControlEvent::Started => handle_started_event().await,
    }
}

async fn load_image<P: AsRef<Path> + Send + Sync + 'static>(path: P) -> Result<RgbaImage> {
    tokio::task::spawn_blocking(move || {
        Ok::<_, anyhow::Error>(image::io::Reader::open(path)?.decode()?.into_rgba8())
    })
    .await?
}

async fn load_image_library() -> Result<ImageLibrary> {
    log::info!("loading images...");
    let (
        dragon_texture_ao_specular_reflection,
        dragon_texture_color,
        dragon_texture_normal,
        plane_texture_color,
        plane_texture_depthmap,
        plane_texture_normal,
        suzanne_texture_ao_specular_reflection,
        suzanne_texture_color,
        suzanne_texture_normal,
    ) = tokio::try_join!(
        load_image("resources/dragon_texture_ao_specular_reflection.png"),
        load_image("resources/dragon_texture_color.png"),
        load_image("resources/dragon_texture_normal.png"),
        load_image("resources/plane_texture_color.png"),
        load_image("resources/plane_texture_depthmap.png"),
        load_image("resources/plane_texture_normal.png"),
        load_image("resources/suzanne_texture_ao_specular_reflection.png"),
        load_image("resources/suzanne_texture_color.png"),
        load_image("resources/suzanne_texture_normal.png"),
    )?;
    log::info!("loaded images");

    Ok(ImageLibrary {
        dragon_texture_ao_specular_reflection,
        dragon_texture_color,
        dragon_texture_normal,
        plane_texture_color,
        plane_texture_depthmap,
        plane_texture_normal,
        suzanne_texture_ao_specular_reflection,
        suzanne_texture_color,
        suzanne_texture_normal,
    })
}

async fn handle_started_event() -> Result<()> {
    let image_library = load_image_library().await?;
    Ok(())
}

fn handle_ui_event(
    window: &Window,
    event: winit::event::Event<ControlToUiEvent>,
    control_flow: &mut ControlFlow,
) -> Result<()> {
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

    Ok(())
}
