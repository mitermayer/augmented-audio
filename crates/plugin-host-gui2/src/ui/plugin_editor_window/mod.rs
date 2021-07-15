//! Module for opening the plugin window.
mod common;
#[cfg(target_os = "macos")]
mod macos;

pub use common::PluginWindowHandle;
#[cfg(target_os = "macos")]
pub use macos::open_plugin_window;
#[cfg(not(target_os = "macos"))]
use vst::editor::Editor;

#[cfg(not(target_os = "macos"))]
pub fn open_plugin_window(mut editor: Box<dyn Editor>, size: (i32, i32)) -> PluginWindowHandle {
    todo!("Not implemented")
}
