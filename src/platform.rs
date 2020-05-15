use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub unsafe fn load_dylib(file: *const c_char, symbols: &mut [(&str, &mut *mut c_void)]) {
    #[cfg(target_family = "unix")]
    let handle = dlopen(file, RTLD_NOW);

    #[cfg(target_family = "windows")]
    let handle = LoadLibraryA(file);

    if handle == std::ptr::null_mut() {
        panic!("load lib {:?}", std::ffi::CStr::from_ptr(file));
    }

    for (name, ptr) in symbols {
        #[cfg(target_family = "unix")]
        let addr = dlsym(handle, c_str!(*name));

        #[cfg(target_os = "windows")]
        let addr = GetProcAddress(handle, c_str!(*name));

        if addr == std::ptr::null_mut() {
            panic!("load fn {} in lib {:?}", name, std::ffi::CStr::from_ptr(file));
        }

        **ptr = addr;
    }
}

pub fn dylib_file(name: &str, ver: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}{}.dll", name, ver)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.{}.dylib", name, ver)
    } else {
        format!("lib{}.so.{}", name, ver)
    }
}

// TODO RTLD_NOW is 0 on android
#[cfg(target_family = "unix")]
const RTLD_NOW: c_int = 2;

#[cfg(target_family = "unix")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[cfg(target_os = "windows")]
extern "C" {
    fn LoadLibraryA(filename: *const c_char) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
}
