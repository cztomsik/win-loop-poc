#[macro_use]
mod macros;
mod nodejs;
mod platform;

use std::os::raw::*;
use crate::nodejs::*;

pub unsafe fn init() {
    std::thread::spawn(move || {
        let uv_loop = uv_default_loop();
        let node_fd = uv_backend_fd(uv_loop);
        assert_ne!(node_fd, -1, "couldnt get uv_loop fd");

        loop {
            std::thread::sleep(std::time::Duration::from_millis(30));

            let mut ev = unsafe { std::mem::zeroed::<kevent>() };
            let timespec = unsafe { std::mem::zeroed::<timespec>() };
            let res = kevent(node_fd, std::ptr::null(), 0, &mut ev, 1, &timespec);

            match res {
                0 => continue,

                -1 => {
                    eprintln!("-- uv polling err --");
                    break;
                }

                // something's pending (res is NOT number of pending events)
                _ => {
                    println!("pending I/O, waking up UI thread");
                    glfwPostEmptyEvent();
                }
            }
        }
    });

    assert_eq!(glfwInit(), GLFW_TRUE);
}

pub unsafe fn wait_event() {
    glfwWaitEvents();
}

pub unsafe fn create_window() {
    glfwCreateWindow(400, 300, c_str!("test"), std::ptr::null_mut(), std::ptr::null_mut());
}


extern {
  fn kevent(kq: c_int, changelist: *const kevent, nchanges: c_int, eventlist: *mut kevent, nevents: c_int, timeout: *const timespec) -> c_int;
}

#[repr(C)]
struct kevent {
    pub ident: usize,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: isize,
    pub udata: *mut c_void,
}

struct timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

// struct without any field is not FFI-safe
pub enum GlfwWindow {}
pub enum GlfwMonitor {}

pub const GLFW_TRUE: c_int = 1;
pub const GLFW_FALSE: c_int = 0;

#[link(name = "glfw3", kind = "static")]
extern "C" {
    fn glfwInit() -> c_int;
    fn glfwCreateWindow(width: c_int, height: c_int, title: *const c_char, monitor: *mut GlfwMonitor, share: *mut GlfwWindow) -> *mut GlfwWindow;
    fn glfwPollEvents();
    fn glfwWaitEvents();
    fn glfwWaitEventsTimeout(timeout: f64);
    fn glfwPostEmptyEvent();

    fn glfwSwapBuffers(window: *mut GlfwWindow);
}

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
#[link(name = "OpenGL", kind = "framework")]
#[link(name = "IOKit", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}
