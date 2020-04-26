#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod settings;

mod bridge;
mod editor;
mod error_handling;
mod keys;
mod redraw_scheduler;
mod renderer;
mod window;

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate lazy_static;

use lazy_static::initialize;

use bridge::BRIDGE;
use std::process;
use window::ui_loop;
use window::window_geometry;

pub const INITIAL_DIMENSIONS: (u64, u64) = (100, 50);

fn main() {
    if let Err(err) = window_geometry() {
        eprintln!("{}", err);
        process::exit(1);
    };
    window::initialize_settings();
    redraw_scheduler::initialize_settings();
    renderer::cursor_renderer::initialize_settings();

    initialize(&BRIDGE);
    ui_loop();
}
