use godot::classes::display_server::HandleType;
use godot::classes::DisplayServer;
use raw_window_handle::{HandleError, HasWindowHandle, RawWindowHandle, WindowHandle};

#[cfg(target_os = "windows")]
use {
    std::num::{NonZero, NonZeroIsize},
    raw_window_handle::{Win32WindowHandle}
};

#[cfg(target_os = "macos")]
use {
    raw_window_handle::{AppKitWindowHandle},
    std::ffi::c_void,
    std::mem::transmute,
    std::ptr::NonNull,
};

#[cfg(target_os = "linux")]
use {
    std::ffi::c_ulong,
    raw_window_handle::{XlibWindowHandle},
};


pub struct GodotWindow;

impl HasWindowHandle for GodotWindow {
    #[cfg(target_os = "windows")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let display_server = DisplayServer::singleton();
        let window_handle = display_server.window_get_native_handle(HandleType::WINDOW_HANDLE);
        let non_zero_window_handle = NonZero::new(window_handle).expect("WindowHandle creation failed");
        unsafe {
            Ok(WindowHandle::borrow_raw(
                RawWindowHandle::Win32(Win32WindowHandle::new({
                    NonZeroIsize::try_from(non_zero_window_handle).expect("Invalid window_handle")
                }))
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let display_server = DisplayServer::singleton();
        let window_handle = display_server.window_get_native_handle(HandleType::WINDOW_VIEW);
        unsafe {
            Ok(WindowHandle::borrow_raw(
                RawWindowHandle::AppKit(AppKitWindowHandle::new({
                    let ptr: *mut c_void = transmute(window_handle);
                    NonNull::new(ptr).expect("Id<T> should never be null")
                }))
            ))
        }
    }

    #[cfg(target_os = "linux")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        use gtk::gdk::prelude::DisplayExtManual;
        use x11_dl::xlib::{Xlib, CWEventMask, SubstructureNotifyMask, SubstructureRedirectMask, XSetWindowAttributes, XWindowAttributes};

        gtk::init().expect("Failed to initialize gtk");
        if !gtk::gdk::Display::default().unwrap().backend().is_x11() {
            panic!("GDK backend must be X11");
        }
        let xlib = Xlib::open().expect("Failed to open Xlib");

        let display_server = DisplayServer::singleton();
        let window_xid = display_server.window_get_native_handle(HandleType::WINDOW_HANDLE);
        let display = display_server.window_get_native_handle(HandleType::DISPLAY_HANDLE);

        unsafe {
            let attributes: XWindowAttributes = std::mem::zeroed();
            let mut attributes = std::mem::MaybeUninit::new(attributes).assume_init();

            let ok = (xlib.XGetWindowAttributes)(
                display as _,
                window_xid as c_ulong,
                &mut attributes,
            );

            if ok != 1 {
                panic!("Failed to get X11 window attributes");
            }

            let mut set_attributes: XSetWindowAttributes = std::mem::zeroed();
            set_attributes.event_mask = attributes.all_event_masks & !SubstructureNotifyMask & !SubstructureRedirectMask;
            let ok = (xlib.XChangeWindowAttributes)(
                display as _,
                window_xid as c_ulong,
                CWEventMask,
                &mut set_attributes,
            );

            if ok != 1 {
                panic!("Failed to change X11 window attributes");
            }
        }

        unsafe {
            Ok(WindowHandle::borrow_raw(
                RawWindowHandle::Xlib(XlibWindowHandle::new({
                    window_xid as c_ulong
                }))
            ))
        }
    }
}
