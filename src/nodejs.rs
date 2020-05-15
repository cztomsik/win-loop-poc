// node.js bindings

#![allow(dead_code)]

use std::os::raw::{c_int, c_uint, c_char, c_void};
use std::ptr;
use std::mem;

dylib! {
    #[load_napi]
    extern "C" {
        fn napi_module_register(module: *mut NapiModule) -> NapiStatus;
        fn napi_get_undefined(env: NapiEnv, result: *mut NapiValue) -> NapiStatus;
        fn napi_set_named_property(env: NapiEnv, object: NapiValue, utf8name: *const c_char, value: NapiValue) -> NapiStatus;
        fn napi_create_function(env: NapiEnv, utf8name: *const c_char, length: usize, cb: NapiCallback, data: *const c_void, result: *mut NapiValue) -> NapiStatus;
        fn napi_get_cb_info(env: NapiEnv, cb_info: NapiCallbackInfo, argc: *mut usize, argv: *mut NapiValue, this_arg: *mut NapiValue, data: *mut c_void) -> NapiStatus;
        fn napi_get_element(env: NapiEnv, arr: NapiValue, index: u32, result: *mut NapiValue) -> NapiStatus;
        fn napi_set_element(env: NapiEnv, arr: NapiValue, index: u32, value: NapiValue) -> NapiStatus;
        fn napi_get_value_uint32(env: NapiEnv, napi_value: NapiValue, result: *mut u32) -> NapiStatus;
        fn napi_get_value_int32(env: NapiEnv, napi_value: NapiValue, result: *mut i32) -> NapiStatus;
        fn napi_get_value_double(env: NapiEnv, napi_value: NapiValue, result: *mut f64) -> NapiStatus;
        fn napi_get_value_bool(env: NapiEnv, napi_value: NapiValue, result: *mut bool) -> NapiStatus;
        fn napi_get_array_length(env: NapiEnv, napi_value: NapiValue, result: *mut u32) -> NapiStatus;
        fn napi_get_value_string_utf8(env: NapiEnv, napi_value: NapiValue, buf: *mut c_char, bufsize: usize, result: *mut usize) -> NapiStatus;
        fn napi_typeof(env: NapiEnv, napi_value: NapiValue, result: *mut NapiValueType) -> NapiStatus;
        fn napi_create_uint32(env: NapiEnv, value: u32, result: *mut NapiValue) -> NapiStatus;
        fn napi_create_int32(env: NapiEnv, value: i32, result: *mut NapiValue) -> NapiStatus;
        fn napi_create_double(env: NapiEnv, value: f64, result: *mut NapiValue) -> NapiStatus;
        fn napi_get_boolean(env: NapiEnv, value: bool, result: *mut NapiValue) -> NapiStatus;
        fn napi_create_array(env: NapiEnv, result: *mut NapiValue) -> NapiStatus;

        fn uv_default_loop() -> *const c_void;
        fn uv_backend_fd(uv_loop: *const c_void) -> c_int;
        fn uv_backend_timeout(uv_loop: *const c_void) -> c_int;
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum NapiStatus {
    Ok,
    InvalidArg,
    ObjectExpected,
    StringExpected,
    NameExpected,
    FunctionExpected,
    NumberExpected,
    BooleanExpected,
    ArrayExpected,
    GenericFailure,
    PendingException,
    Cancelled,
    EscapeCalledTwice,
    HandleScopeMismatch,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum NapiValueType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    Object,
    Function,
    External,
    Bigint,
}

#[repr(C)]
pub struct NapiModule {
    nm_version: c_int,
    nm_flags: c_uint,
    nm_filename: *const c_char,
    nm_register_func: unsafe extern "C" fn(NapiEnv, NapiValue) -> NapiValue,
    nm_modname: *const c_char,
    nm_priv: *const c_void,
    reserved: [*const c_void; 4],
}

pub type NapiCallback = unsafe extern "C" fn(NapiEnv, NapiCallbackInfo) -> NapiValue;
const NAPI_AUTO_LENGTH: usize = usize::max_value();

// opaque types
#[derive(Clone, Copy)] #[repr(C)] pub struct NapiValue(*const c_void);
#[derive(Clone, Copy)] #[repr(C)] pub struct NapiEnv(*const c_void);
#[repr(C)] pub struct NapiCallbackInfo(*const c_void);

#[no_mangle]
#[cfg_attr(target_os = "linux", link_section = ".ctors")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
pub static REGISTER_NODE_MODULE: unsafe extern "C" fn() = {
    static mut NAPI_MODULE: Option<NapiModule> = None;

    unsafe extern "C" fn register_node_module() {
        println!("loading napi");

        // can't use ternary because of c_str!
        #[cfg(target_family = "unix")]
        load_napi(std::ptr::null());
        #[cfg(target_os = "windows")]
        load_napi(c_str!("node.exe"));

        NAPI_MODULE = Some(NapiModule {
            nm_version: 1,
            nm_flags: 0,
            nm_filename: c_str!(file!()),
            nm_register_func: init_node_module,
            nm_modname: c_str!("libgraffiti"),
            nm_priv: ptr::null(),
            reserved: [ptr::null(); 4]
        });

        println!("registering napi module");
        napi_module_register(NAPI_MODULE.as_mut().unwrap() as *mut NapiModule);
    }

    register_node_module    
};

// - call napi fn with env & uninitialized mem space for the result
// - check if it was ok
// - return the result
macro_rules! get_res {
    ($napi_fn:ident $($arg:tt)*) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let mut res_value = mem::MaybeUninit::uninit().assume_init();
            let res = $napi_fn(ENV $($arg)*, &mut res_value);

            assert_eq!(res, NapiStatus::Ok);

            res_value
        }
    }}
}

unsafe extern "C" fn init_node_module(env: NapiEnv, exports: NapiValue) -> NapiValue {
    println!("initializing app");

    super::init();

    ENV = env;

    let method = get_res!(napi_create_function, std::ptr::null(), NAPI_AUTO_LENGTH, js_create_window, ptr::null());
    napi_set_named_property(env, exports, c_str!("createWindow"), method);

    let method = get_res!(napi_create_function, std::ptr::null(), NAPI_AUTO_LENGTH, js_wait_event, ptr::null());
    napi_set_named_property(env, exports, c_str!("waitEvent"), method);

    exports
}

unsafe extern "C" fn js_wait_event(env: NapiEnv, cb_info: NapiCallbackInfo) -> NapiValue {
    super::wait_event();

    get_res!(napi_get_undefined)
}

unsafe extern "C" fn js_create_window(env: NapiEnv, cb_info: NapiCallbackInfo) -> NapiValue {
    super::create_window();

    get_res!(napi_get_undefined)
}

static mut ENV: NapiEnv = NapiEnv(ptr::null_mut());
