use std::time::{Duration, Instant};

use std::sync::atomic::Ordering;
use image::{load_from_memory, GenericImageView, Pixel};
use skulpin::{
    CoordinateSystem,
    RendererBuilder,
    PresentMode,
    PhysicalSize,
    Renderer as SkulpinRenderer,
};
use skulpin::winit;
use skulpin::winit::dpi::{LogicalSize};
use skulpin::winit::event::{ElementState, Event, MouseScrollDelta, StartCause, WindowEvent};
use skulpin::winit::event_loop::{ControlFlow, EventLoop};
use skulpin::winit::window::{Icon, WindowBuilder};
use log::{info, debug, trace, error};

use crate::bridge::{BRIDGE, UiCommand};
use crate::renderer::Renderer;
use crate::redraw_scheduler::REDRAW_SCHEDULER;
use crate::settings::*;
use crate::INITIAL_DIMENSIONS;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn handle_new_grid_size(new_size: LogicalSize<f64>, renderer: &Renderer) {
    if new_size.width > 0. && new_size.height > 0. {
        let new_width = ((new_size.width + 1.) as f32 / renderer.font_width) as u32;
        let new_height = ((new_size.height + 1.) as f32 / renderer.font_height) as u32;
        // Add 1 here to make sure resizing doesn't change the grid size on startup
        BRIDGE.queue_command(UiCommand::Resize {
            width: new_width,
            height: new_height,
        });
    }
}

struct WindowWrapper {
    window: winit::window::Window,
    skulpin_renderer: SkulpinRenderer,
    renderer: Renderer,
    mouse_down: bool,
    mouse_position: skulpin::LogicalSize,
}

pub fn window_geometry() -> Result<(u64, u64), String> {
    let prefix = "--geometry=";

    std::env::args()
        .filter(|arg| arg.starts_with(prefix))
        .next()
        .map_or(Ok(INITIAL_DIMENSIONS), |arg| {
            let input = &arg[prefix.len()..];
            let invalid_parse_err = format!(
                "Invalid geometry: {}\nValid format: <width>x<height>",
                input
            );

            input
                .split('x')
                .map(|dimension| {
                    dimension
                        .parse::<u64>()
                        .or(Err(invalid_parse_err.as_str()))
                        .and_then(|dimension| {
                            if dimension > 0 {
                                Ok(dimension)
                            } else {
                                Err("Invalid geometry: Window dimensions should be greater than 0.")
                            }
                        })
                })
                .collect::<Result<Vec<_>, &str>>()
                .and_then(|dimensions| {
                    if let [width, height] = dimensions[..] {
                        Ok((width, height))
                    } else {
                        Err(invalid_parse_err.as_str())
                    }
                })
                .map_err(|msg| msg.to_owned())
        })
}

pub fn window_geometry_or_default() -> (u64, u64) {
    window_geometry().unwrap_or(INITIAL_DIMENSIONS)
}

impl WindowWrapper {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> WindowWrapper {
        let renderer = Renderer::new();

        let icon = {
            let icon_data = Asset::get("nvim.ico").expect("Failed to read icon data");
            let icon = load_from_memory(&icon_data).expect("Failed to parse icon data");
            let (width, height) = icon.dimensions();
            let mut rgba = Vec::with_capacity((width * height) as usize * 4);
            for (_, _, pixel) in icon.pixels() {
                rgba.extend_from_slice(&pixel.to_rgba().0);
            }
            Icon::from_rgba(rgba, width, height).expect("Failed to create icon object")
        };
        info!("icon created");

        let title = "Neovide";
        let winit_window = WindowBuilder::new()
            .with_title(title)
            .with_window_icon(Some(icon))
            .build(event_loop)
            .expect("Failed to create window");
        info!("window created");

        let window = skulpin::WinitWindow::new(&winit_window);
        let skulpin_renderer = RendererBuilder::new()
            .use_vulkan_debug_layer(true)
            .present_mode_priority(vec![PresentMode::Mailbox, PresentMode::Immediate])
            .coordinate_system(CoordinateSystem::Logical)
            .build(&window)
            .expect("Failed to create renderer");
        info!("renderer created");

        WindowWrapper {
            window: winit_window,
            skulpin_renderer,
            renderer,
            mouse_down: false,
            mouse_position: skulpin::LogicalSize {
                width: 0,
                height: 0,
            },
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.window.fullscreen() == None {
            // TODO self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(xxx)));
        } else {
            self.window.set_fullscreen(None)
        }
    }

    pub fn synchronize_settings(&mut self) {
        // TODO not working very well

        //let editor_title = { EDITOR.lock().title.clone() };
        //self.window.set_title(&editor_title);

        //let transparency = { SETTINGS.get::<WindowSettings>().transparency };
        //if let Ok(opacity) = self.window.opacity() {
            // TODO for winit?
            //if opacity != transparency {
            //    self.window.set_opacity(transparency).ok();
            //    self.transparency = transparency;
            //}
        //}

        let fullscreen = { SETTINGS.get::<WindowSettings>().fullscreen };

        if (self.window.fullscreen() != None) != fullscreen {
            self.toggle_fullscreen();
        }
    }

    pub fn handle_quit(&mut self) {
        BRIDGE.queue_command(UiCommand::Quit);
    }

    pub fn handle_keyboard_input(&self, input: String) {
        BRIDGE.queue_command(UiCommand::Keyboard(input));
    }

    pub fn handle_pointer_motion(&mut self, x: u32, y: u32) {
        let previous_position = self.mouse_position;
        let physical_size = PhysicalSize::new(
            (x as f32 / self.renderer.font_width) as u32,
            (y as f32 / self.renderer.font_height) as u32,
        );

        self.mouse_position = physical_size.to_logical(self.window.scale_factor());
        if self.mouse_down && previous_position != self.mouse_position {
            BRIDGE.queue_command(UiCommand::Drag(
                self.mouse_position.width as u32,
                self.mouse_position.height as u32,
            ));
        }
    }

    pub fn handle_pointer_down(&mut self) {
        BRIDGE.queue_command(UiCommand::MouseButton {
            action: String::from("press"),
            position: (self.mouse_position.width, self.mouse_position.height),
        });
        self.mouse_down = true;
    }

    pub fn handle_pointer_up(&mut self) {
        BRIDGE.queue_command(UiCommand::MouseButton {
            action: String::from("release"),
            position: (self.mouse_position.width, self.mouse_position.height),
        });
        self.mouse_down = false;
    }

    pub fn handle_mouse_wheel(&mut self, x: f32, y: f32) {
        let vertical_input_type = if y > 0.0 {
            Some("up")
        } else if y < 0.0 {
            Some("down")
        } else {
            None
        };

        if let Some(input_type) = vertical_input_type {
            BRIDGE.queue_command(UiCommand::Scroll {
                direction: input_type.to_string(),
                position: (self.mouse_position.width, self.mouse_position.height),
            });
        }

        let horizontal_input_type = if x > 0.0 {
            Some("right")
        } else if x < 0.0 {
            Some("left")
        } else {
            None
        };

        if let Some(input_type) = horizontal_input_type {
            BRIDGE.queue_command(UiCommand::Scroll {
                direction: input_type.to_string(),
                position: (self.mouse_position.width, self.mouse_position.height),
            });
        }
    }

    pub fn handle_focus_lost(&mut self) {
        BRIDGE.queue_command(UiCommand::FocusLost);
    }

    pub fn handle_focus_gained(&mut self) {
        BRIDGE.queue_command(UiCommand::FocusGained);
        REDRAW_SCHEDULER.queue_next_frame();
    }

    pub fn draw_frame(&mut self) -> bool {
        if !BRIDGE.running.load(Ordering::Relaxed) {
            return false;
        }

        let window = skulpin::WinitWindow::new(&self.window);

        debug!("Render Triggered");

        if REDRAW_SCHEDULER.should_draw() || SETTINGS.get::<WindowSettings>().no_idle {
            let renderer = &mut self.renderer;

            let size = self.window.inner_size().to_logical(self.window.scale_factor());

            if self.skulpin_renderer.draw(&window, |canvas, coordinate_system_helper| {
                let dt = 1.0 / (SETTINGS.get::<WindowSettings>().refresh_rate as f32);

                if renderer.draw(canvas, &coordinate_system_helper, dt) {
                    handle_new_grid_size(size, renderer);
                }
            }).is_err()
            {
                error!("Render failed.");
                return false;
            }
        }

        return true;
    }
}

#[derive(Clone)]
struct WindowSettings {
    refresh_rate: u64,
    transparency: f32,
    no_idle: bool,
    fullscreen: bool,
}

pub fn initialize_settings() {
    let no_idle = SETTINGS
        .neovim_arguments
        .contains(&String::from("--noIdle"));

    SETTINGS.set(&WindowSettings {
        refresh_rate: 60,
        transparency: 1.0,
        no_idle,
        fullscreen: false,
    });

    register_nvim_setting!("refresh_rate", WindowSettings::refresh_rate);
    register_nvim_setting!("transparency", WindowSettings::transparency);
    register_nvim_setting!("no_idle", WindowSettings::no_idle);
    register_nvim_setting!("fullscreen", WindowSettings::fullscreen);
}

pub fn ui_loop() {
    let event_loop = EventLoop::<()>::with_user_event();

    let mut window = WindowWrapper::new(&event_loop);

    event_loop.run(move |event, _window_target, control_flow| {
        trace!("Window Event: {:?}", event);
        match event {
            Event::NewEvents(StartCause::Init) |
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                window.window.request_redraw()
            },

            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        window.handle_quit();
                        *control_flow = ControlFlow::Exit;
                    },

                    WindowEvent::Resized(new_size) => {
                        handle_new_grid_size(new_size.to_logical(window.window.scale_factor()), &window.renderer)
                    },

                    WindowEvent::ReceivedCharacter(c) => {
                        window.handle_keyboard_input(
                            match c {
                                '<' => "<lt>".to_string(),
                                _ => c.to_string()
                            }
                        );
                    },

                    WindowEvent::CursorMoved { position, .. } => {
                        if position.x >= 0.0 && position.y >= 0.0 {
                            window.handle_pointer_motion(position.x as u32, position.y as u32);
                        }
                    },

                    WindowEvent::MouseInput { state, .. } => {
                        match state {
                            ElementState::Pressed => {
                                window.handle_pointer_down();
                            },
                            ElementState::Released => {
                                window.handle_pointer_up();
                            },
                        };
                    },
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(horizontal, vertical),
                        ..
                    } => {
                        window.handle_mouse_wheel(horizontal, vertical);
                    },

                    WindowEvent::Focused(focused) => {
                        if focused {
                            window.handle_focus_gained();
                        } else {
                            window.handle_focus_lost();
                        }
                    },

                    WindowEvent::DroppedFile(path) => {
                        if let Some(valid_str) = path.to_str() {
                            BRIDGE.queue_command(UiCommand::FileDrop(valid_str.to_string()));
                        }
                    }

                    _ => ()
                }
            }

            Event::RedrawRequested { .. } => {
                let frame_start = Instant::now();
                let refresh_rate = { SETTINGS.get::<WindowSettings>().refresh_rate as f32 };
                let frame_length = Duration::from_secs_f32(1.0 / refresh_rate);

                if window.draw_frame() {
                    *control_flow = ControlFlow::WaitUntil(frame_start + frame_length);
                } else {
                    // XXX this is propably not right way to exit
                    std::process::exit(0);
                }
            },

            _ => {}
        }

        window.synchronize_settings();
    });
}
