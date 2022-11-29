mod assets;
mod entity;
mod gfx;

use anyhow::Result;
use assets::Model;
use dotenv::dotenv;
use entity::{EntityBuilder, Transform, World};
use gfx::{gl::GfxGlBackend, GfxBackend};
use image::RgbaImage;
use nalgebra::Vector3;
use tokio::sync::mpsc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

use crate::assets::{load_image, load_model};

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

pub struct ModelLibrary {
    pub dragon: Model,
    pub plane: Model,
    pub suzanne: Model,
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
    run_with::<GfxGlBackend>()
}

fn run_with<B>() -> Result<()>
where
    B: GfxBackend,
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

async fn load_model_library() -> Result<ModelLibrary> {
    log::info!("loading models...");
    let (dragon, plane, suzanne) = tokio::try_join!(
        load_model("resources/dragon.obj"),
        load_model("resources/plane.obj"),
        load_model("resources/suzanne.obj"),
    )?;
    log::info!("loaded models");
    Ok(ModelLibrary {
        dragon,
        plane,
        suzanne,
    })
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

fn build_scene() -> Result<()> {
    let mut world = World::default();
    // camera
    world.add(
        EntityBuilder::default()
            .transform(Transform::default())
            .camera(Default::default())
            .build(),
    );
    // skybox
    world.add(EntityBuilder::default().build());
    // suzanne
    world.add(EntityBuilder::default().build());
    // dragon
    world.add(
        EntityBuilder::default()
            .transform(
                Transform::default()
                    .translate(Vector3::new(-0.1, -0.05, -0.25))
                    .scale(0.5),
            )
            .build(),
    );
    // plane
    world.add(
        EntityBuilder::default()
            .transform(
                Transform::default()
                    .translate(Vector3::new(0.0, -0.35, -0.5))
                    .scale(2.0),
            )
            .build(),
    );
    Ok(())
}

async fn handle_started_event() -> Result<()> {
    let (image_library, model_library) =
        tokio::try_join!(load_image_library(), load_model_library())?;
    build_scene()?;
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
