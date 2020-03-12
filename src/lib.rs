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

impl std::fmt::Display for Message {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_ref())
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}

pub struct MemoryBuffer(NonNull<llvm::LLVMMemoryBuffer>);

llvm_inner_impl!(MemoryBuffer, llvm::LLVMMemoryBuffer);

impl MemoryBuffer {
    pub(crate) fn from_raw(ptr: *mut llvm::LLVMMemoryBuffer) -> Result<Self, Error> {
        Ok(MemoryBuffer(wrap_inner(ptr)?))
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<MemoryBuffer, Error> {
        let path = match path.as_ref().to_str() {
            Some(p) => cstr!(p),
            None => return Err(Error::InvalidPath),
        };

        let mut mem = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();

        let ok = unsafe {
            llvm::core::LLVMCreateMemoryBufferWithContentsOfFile(
                path.as_ptr(),
                &mut mem,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if !ok {
            return Err(Error::Message(message));
        }

        Self::from_raw(mem)
    }

    pub fn from_slice(name: impl AsRef<str>, s: impl AsRef<[u8]>) -> Result<MemoryBuffer, Error> {
        let name = cstr!(name.as_ref());
        let s = s.as_ref();
        let mem = unsafe {
            llvm::core::LLVMCreateMemoryBufferWithMemoryRangeCopy(
                s.as_ptr() as *const c_char,
                s.len(),
                name.as_ptr(),
            )
        };

        Self::from_raw(mem)
    }

    pub fn size(&self) -> usize {
        unsafe { llvm::core::LLVMGetBufferSize(self.0.as_ptr()) }
    }

    pub fn as_slice(&self) -> &[u8] {
        let size = self.size();
        unsafe {
            let data = llvm::core::LLVMGetBufferStart(self.0.as_ptr());
            std::slice::from_raw_parts(data as *const u8, size)
        }
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeMemoryBuffer(self.0.as_ptr()) }
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
