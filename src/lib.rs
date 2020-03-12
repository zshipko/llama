#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(unused)]
mod wavm_bindings;

macro_rules! cstr {
    ($x:expr) => {
        std::ffi::CString::new($x).expect("Invalid C string")
    };
}

macro_rules! llvm_inner_impl {
    ($t:ty, $u:ty) => {
        impl<'a> LLVMInner<$u> for $t {
            fn llvm_inner(&self) -> *mut $u {
                self.0.as_ptr()
            }
        }
    };
}

extern "C" {
    fn strlen(_: *const std::os::raw::c_char) -> usize;
}

mod basic_block;
mod builder;
mod context;
mod error;
mod module;
mod typ;
mod value;

pub(crate) use std::ffi::c_void;
pub(crate) use std::marker::PhantomData;
pub(crate) use std::os::raw::{c_char, c_int, c_uint};
pub(crate) use std::ptr::NonNull;

pub(crate) use llvm_sys as llvm;

pub use crate::basic_block::BasicBlock;
pub use crate::builder::Builder;
pub use crate::context::Context;
pub use crate::error::Error;
pub use crate::module::Module;
pub use crate::typ::{Type, TypeKind};
pub use crate::value::{Value, ValueKind};

pub trait LLVMInner<T> {
    fn llvm_inner(&self) -> *mut T;
}

pub(crate) fn wrap_inner<T>(x: *mut T) -> Result<NonNull<T>, Error> {
    match NonNull::new(x) {
        Some(x) => Ok(x),
        None => Err(Error::NullPointer),
    }
}

pub struct Message(*mut c_char);
impl Message {
    pub(crate) fn from_raw(c: *mut c_char) -> Message {
        Message(c)
    }

    pub fn len(&self) -> usize {
        if self.0.is_null() {
            return 0;
        }

        unsafe { strlen(self.0) }
    }
}

impl AsRef<str> for Message {
    fn as_ref(&self) -> &str {
        if self.0.is_null() {
            return "<NULL>";
        }

        unsafe {
            let st = std::slice::from_raw_parts(self.0 as *const u8, self.len());
            std::str::from_utf8_unchecked(st)
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { llvm::core::LLVMDisposeMessage(self.0) }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() {
        let context = Context::new().unwrap();
        let module = Module::new(&context, "testing").unwrap();
        let i32 = Type::int(&context, 32);
        assert_eq!(module.identifier().unwrap(), "testing");
    }
}
