// Desktop / iOS arms all pull a native handle off DisplayServer and wrap it in
// a RawWindowHandle variant. The Android arm returns an Unavailable stub (the
// real ANativeWindow comes from the Phase 4 plugin shim, not DisplayServer),
// so these imports are dead on Android.
#[cfg(not(target_os = "android"))]
use godot::classes::display_server::HandleType;
#[cfg(not(target_os = "android"))]
use godot::classes::DisplayServer;
#[cfg(not(target_os = "android"))]
use godot::obj::Singleton;
use raw_window_handle::{HandleError, HasWindowHandle, WindowHandle};
#[cfg(not(target_os = "android"))]
use raw_window_handle::RawWindowHandle;

#[cfg(target_os = "windows")]
use {
    std::num::{NonZero, NonZeroIsize},
    raw_window_handle::{Win32WindowHandle}
};

#[cfg(target_os = "macos")]
use {
    raw_window_handle::{AppKitWindowHandle},
    std::ffi::c_void,
    std::ptr::NonNull,
};

#[cfg(target_os = "linux")]
use {
    std::ffi::c_ulong,
    raw_window_handle::{XlibWindowHandle},
};

#[cfg(target_os = "ios")]
use {
    raw_window_handle::UiKitWindowHandle,
    std::ffi::c_void,
    std::ptr::NonNull,
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
                    // `with_exposed_provenance_mut` is the explicit, lint-clean
                    // alternative to `transmute` / `as *mut T` for int -> ptr.
                    // Godot returned the NSView address in `window_handle`.
                    let ptr: *mut c_void = std::ptr::with_exposed_provenance_mut(window_handle as usize);
                    NonNull::new(ptr).expect("Id<T> should never be null")
                }))
            ))
        }
    }

    // Phase 1 stub: compiles for iOS so the static lib + xcframework can link
    // into Godot iOS export. The real implementation that pulls a UIView from
    // the iOS GDExtension plugin shim lives in Phase 2.
    #[cfg(target_os = "ios")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let display_server = DisplayServer::singleton();
        let window_handle = display_server.window_get_native_handle(HandleType::WINDOW_VIEW);
        if window_handle == 0 {
            return Err(HandleError::Unavailable);
        }
        unsafe {
            let ptr: *mut c_void = std::ptr::with_exposed_provenance_mut(window_handle as usize);
            let nn = NonNull::new(ptr).ok_or(HandleError::Unavailable)?;
            Ok(WindowHandle::borrow_raw(
                RawWindowHandle::UiKit(UiKitWindowHandle::new(nn))
            ))
        }
    }

    // Phase 3 stub: compiles for Android so the .so links into Godot Android
    // export. wry's Android path uses the `android_setup!()` macro and an
    // ANativeWindow obtained from the host Activity rather than this raw
    // window handle. The real plugin shim that bridges Activity / JavaVM and
    // wires up `android_setup!()` lives in Phase 4.
    #[cfg(target_os = "android")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
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
