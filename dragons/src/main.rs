mod assets;

use std::{ffi::CString, num::NonZeroU32};

use anyhow::Result;
use assets::Model;
use dotenv::dotenv;
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use image::RgbaImage;
use raw_window_handle::HasRawWindowHandle;
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
    dotenv().ok();
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let window_width = 1280f64;
    let window_height = 720f64;
    let event_loop: EventLoop<ControlToUiEvent> = EventLoopBuilder::with_user_event().build();

    let window_builder = WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            window_width,
            window_height,
        )))
        .with_title("Demo");
    let template_builder = ConfigTemplateBuilder::new().with_depth_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let (window, gl_config) = display_builder
        .build(&event_loop, template_builder, |mut configs| {
            configs.next().unwrap()
        })
        .expect("cannot find proper window config");
    let window = window.expect("window must be valid");
    let raw_window_handle = window.raw_window_handle();

    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

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

    let gl_window = GlWindow::new(window, &gl_config);
    let gl_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&gl_window.surface)
        .unwrap();

    gl_window
        .surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .unwrap();

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::RedrawEventsCleared => unsafe {
                gl::ClearColor(0.0, 0.5, 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl_window.window.request_redraw();
                gl_window.surface.swap_buffers(&gl_context).unwrap();
            },
            Event::Resumed => {
                // Make it current.
            }
            Event::Suspended => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        gl_window.surface.resize(
                            &gl_context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                _ => (),
            },
            _ => (),
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
async fn handle_started_event() -> Result<()> {
    let (image_library, model_library) =
        tokio::try_join!(load_image_library(), load_model_library())?;
    Ok(())
}

fn handle_ui_event(
    window: &Window,
    event: winit::event::Event<ControlToUiEvent>,
    control_flow: &mut ControlFlow,
) -> Result<()> {
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

pub struct GlWindow {
    // XXX the surface must be dropped before the window.
    pub surface: Surface<WindowSurface>,

    pub window: Window,
}

impl GlWindow {
    pub fn new(window: Window, config: &Config) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            config
                .display()
                .create_window_surface(config, &attrs)
                .unwrap()
        };

        Self { window, surface }
    }
}
