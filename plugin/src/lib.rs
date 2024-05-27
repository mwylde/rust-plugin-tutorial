use std::ffi::{CStr, CString};
use std::panic::catch_unwind;

// An FFI-safe value enum to support various input/output types
#[repr(C)]
pub enum PluginValue {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Double(f64),
    // Strings are represented as a pointer to a null-terminated string; all strings are owned
    // by the host. Returned strings must be freed by the host.
    String(*const i8),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PluginType {
    Bool,
    Int,
    UInt,
    Double,
    String,
}

// An FFI-safe result type
#[repr(C)]
pub enum PluginResult {
    Ok(PluginValue),
    // The host is responsible for freeing the error message
    Err(*mut i8),
}

#[repr(C)]
pub struct PluginMetadata {
    pub name: *const i8,
    pub arg_types: *const PluginType,
    pub arg_types_len: usize,
    pub return_type: PluginType,
}

// The metadata function that will be called by the host to get information about the plugin.
#[no_mangle]
pub extern "C" fn plugin_metadata() -> PluginMetadata {
    PluginMetadata {
        name: "repeat\0".as_ptr() as *const i8,
        arg_types: [PluginType::String, PluginType::UInt].as_ptr(),
        arg_types_len: 2,
        return_type: PluginType::String,
    }
}

fn plugin_error(message: impl Into<String>) -> PluginResult {
    PluginResult::Err(CString::new(message.into()).unwrap().into_raw())
}

// The main plugin function that will be called by the host. It is annotated with #[no_mangle] to
// prevent the Rust compiler from mangling the name of the function. All arguments and return values
// must be FFI safe types.
//
// This function wraps the actual implementation, validating and converting the arguments, then
// catching any panics that occur in the implementation. All unsafe (and corresponding care around
// ensuring safety) is contained in this method, allowing the actual implementation to be normal,
// safe Rust code.
//
// In a real plugin system, you would likely want to generate this function using a macro to avoid
// the boilerplate.
#[no_mangle]
pub extern "C" fn plugin_entrypoint(args: *const PluginValue, args_len: usize) -> PluginResult {
    // first we need to check if the arguments are valid
    if args_len != 2 {
        return plugin_error("args_len should be 2");
    }

    let PluginValue::String(string) = (unsafe { &*args.offset(0) }) else {
        return plugin_error("arg0 is invalid; expected String");
    };

    let PluginValue::UInt(count) = (unsafe { &*args.offset(1) }) else {
        return plugin_error("arg1 is invalid; expected UInt");
    };

    let string = match unsafe { CStr::from_ptr(*string) }.to_str() {
        Ok(value) => value,
        Err(_) => {
            return plugin_error("arg0 is invalid; expected valid UTF-8 string");
        }
    };

    match catch_unwind(|| repeat_impl(string, *count)) {
        Ok(value) => PluginResult::Ok(PluginValue::String(CString::new(value).unwrap().into_raw())),
        Err(_) => plugin_error("function panicked"),
    }
}

// The actual implementation of the plugin function. This is a normal Rust function that can be
// tested and used in other Rust code.
fn repeat_impl(arg1: &str, arg2: u64) -> String {
    arg1.repeat(arg2 as usize)
}
