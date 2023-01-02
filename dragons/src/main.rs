mod assets;
mod gl_utils;
mod scene;

use std::{ffi::CString, fs::read_to_string, num::NonZeroU32, path::PathBuf};

use anyhow::Result;
use dotenv::dotenv;
use gl_utils::mesh::Mesh;
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, NotCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use scene::Scene;
use tokio::sync::mpsc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

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
    let (gl_window, mut maybe_gl_context) = init_gl_window(&event_loop, window_builder)?;

    let gl_context = maybe_gl_context
        .take()
        .unwrap()
        .make_current(&gl_window.surface)
        .unwrap();

    gl_window
        .surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
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

    let resources_path = "resources";
    let scene_path: PathBuf = [resources_path, "scene.ron"].iter().collect();

    let scene_ron = read_to_string(scene_path)?;
    let scene: Scene = ron::from_str(&scene_ron)?;
    let meshes = scene
        .objects
        .values()
        .map(|obj| {
            let mesh_path: PathBuf = [resources_path, &obj.mesh].iter().collect();
            Mesh::from_obj(&gl_context, mesh_path)
        })
        .collect::<Result<Vec<_>, _>>()?;

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

async fn handle_started_event() -> Result<()> {
    Ok(())
}

fn init_gl_window<T: 'static>(
    event_loop: &EventLoopWindowTarget<T>,
    window_builder: WindowBuilder,
) -> Result<(GlWindow, Option<NotCurrentContext>)> {
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
    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let maybe_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

    Ok((GlWindow::new(window, &gl_config), maybe_gl_context))
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
