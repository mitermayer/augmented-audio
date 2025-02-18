use std::ffi::c_void;

use cocoa::appkit::{NSView, NSWindow, NSWindowStyleMask};
use cocoa::appkit::NSBackingStoreType::NSBackingStoreBuffered;
use cocoa::base::{id, nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use iced::{Point, Rectangle, Size};
use objc::runtime::Object;
use raw_window_handle::macos::MacOSHandle;
use raw_window_handle::RawWindowHandle;
use vst::editor::Editor;

use super::PluginWindowHandle;

pub fn open_plugin_window(
    editor: &mut Box<dyn Editor>,
    size: (i32, i32),
    position: Option<Point>,
) -> PluginWindowHandle {
    let _pool = unsafe { NSAutoreleasePool::new(nil) };
    let (width, height) = size;
    let rect = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(width as f64, height as f64),
    );
    let ns_window = unsafe {
        let ns_window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            rect,
            NSWindowStyleMask::NSTitledWindowMask,
            NSBackingStoreBuffered,
            NO,
        );
        // .autorelease();
        ns_window.center();

        let title = NSString::alloc(nil).init_str("plugin-window"); // .autorelease();
        ns_window.setTitle_(title);

        ns_window.makeKeyAndOrderFront_(nil);

        ns_window
    };
    let ns_view = unsafe { ns_window.contentView() };
    let raw_window_handle = RawWindowHandle::MacOS(MacOSHandle {
        ns_window: ns_window as *mut c_void,
        ns_view: ns_view as *mut c_void,
        ..MacOSHandle::empty()
    });
    editor.open(ns_view as *mut c_void);

    if let Some(position) = position {
        log::info!("Restoring plugin window position");
        let ns_point = NSPoint {
            x: position.x as f64,
            y: position.y as f64,
        };
        unsafe {
            NSWindow::setFrameTopLeftPoint_(ns_window, ns_point);
        }
    }

    PluginWindowHandle { raw_window_handle }
}

pub fn close_window(handle: RawWindowHandle) -> Option<Rectangle> {
    if let RawWindowHandle::MacOS(MacOSHandle {
        ns_window, ns_view, ..
    }) = handle
    {
        unsafe {
            let ns_window = ns_window as id;
            let ns_view = ns_view as id;
            let window_frame = get_window_frame(ns_window);

            ns_view.removeFromSuperview();
            ns_window.close();
            let _ = Box::from_raw(ns_view);
            let _ = Box::from_raw(ns_window);

            return Some(window_frame);
        }
    }

    None
}

unsafe fn get_window_frame(ns_window: *mut Object) -> Rectangle {
    let frame = NSWindow::frame(ns_window);
    let bottom_left = frame.origin;
    let size = frame.size;
    Rectangle::new(
        Point::new(bottom_left.x as f32, (bottom_left.y + size.height) as f32),
        Size::new(size.width as f32, size.height as f32),
    )
}

pub fn float_window(handle: &RawWindowHandle) {
    if let RawWindowHandle::MacOS(MacOSHandle { ns_window, .. }) = handle {
        unsafe {
            let window = *ns_window as id;
            window.setLevel_(3);
        }
    }
}
