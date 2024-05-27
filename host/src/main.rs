use dlopen2::wrapper::{Container, WrapperApi};
use std::env::args;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};

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

impl PluginValue {
    pub fn to_owned(self) -> OwnedPluginValue {
        match self {
            PluginValue::Bool(b) => OwnedPluginValue::Bool(b),
            PluginValue::Int(i) => OwnedPluginValue::Int(i),
            PluginValue::UInt(u) => OwnedPluginValue::UInt(u),
            PluginValue::Double(d) => OwnedPluginValue::Double(d),
            PluginValue::String(s) => {
                OwnedPluginValue::String(unsafe { CString::from_raw(s as *mut i8) })
            }
        }
    }
}

// An owned version of PluginValue that owns all dynamically allocated resources,
// such that memory will be freed when the value is dropped.
pub enum OwnedPluginValue {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Double(f64),
    String(CString),
}

impl Display for OwnedPluginValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnedPluginValue::Bool(b) => write!(f, "{}", b),
            OwnedPluginValue::Int(i) => write!(f, "{}", i),
            OwnedPluginValue::UInt(u) => write!(f, "{}", u),
            OwnedPluginValue::Double(d) => write!(f, "{}", d),
            OwnedPluginValue::String(s) => write!(f, "{}", s.to_string_lossy()),
        }
    }
}

// An FFI-safe result type
#[repr(C)]
pub enum PluginResult {
    Ok(PluginValue),
    // The host is responsible for freeing the error message
    Err(*mut i8),
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

#[repr(C)]
pub struct PluginMetadata {
    pub name: *const i8,
    pub arg_types: *const PluginType,
    pub arg_types_len: usize,
    pub return_type: PluginType,
}

#[derive(WrapperApi)]
struct PluginApi {
    plugin_metadata: unsafe extern "C" fn() -> PluginMetadata,
    plugin_entrypoint:
        unsafe extern "C" fn(args: *const PluginValue, args_len: usize) -> PluginResult,
}

fn main() {
    let args: Vec<_> = args().into_iter().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <plugin>", args[0]);
        std::process::exit(1);
    }

    let container: Container<PluginApi> =
        unsafe { Container::load(&args[1]) }.expect("Could not load plugin");

    let metadata: PluginMetadata = unsafe { container.plugin_metadata() };
    println!("Loaded plugin {}", unsafe {
        CStr::from_ptr(metadata.name).to_string_lossy()
    });

    if metadata.arg_types_len != args.len() - 2 {
        eprintln!(
            "Expected {} arguments, got {}",
            metadata.arg_types_len,
            args.len() - 2
        );
        std::process::exit(1);
    }

    let mut call_args: Vec<PluginValue> = vec![];
    for (i, arg) in args[2..].iter().enumerate() {
        match unsafe { *metadata.arg_types.add(i) } {
            PluginType::Bool => {
                call_args.push(PluginValue::Bool(arg.parse().expect("Invalid bool")))
            }
            PluginType::Int => call_args.push(PluginValue::Int(arg.parse().expect("Invalid int"))),
            PluginType::UInt => {
                call_args.push(PluginValue::UInt(arg.parse().expect("Invalid uint")))
            }
            PluginType::Double => {
                call_args.push(PluginValue::Double(arg.parse().expect("Invalid double")))
            }
            PluginType::String => call_args.push(PluginValue::String(arg.as_ptr() as *const i8)),
        }
    }

    let result = unsafe { container.plugin_entrypoint(call_args.as_ptr(), call_args.len()) };

    // take ownership of the plugin arguments and drop them, ensuring that any dynamically allocated
    // memory is freed
    drop(call_args.into_iter().map(|t| t.to_owned()));

    match result {
        PluginResult::Ok(value) => {
            println!("Plugin returned: {}", value.to_owned());
        }
        PluginResult::Err(err) => {
            eprintln!("{}", unsafe { CString::from_raw(err) }.to_string_lossy());
            std::process::exit(1);
        }
    }
}
