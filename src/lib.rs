#![allow(clippy::should_implement_trait)]
//#![deny(missing_docs)]

//! `llama` is a friendly LLVM wrapper
//!
//! # Getting started
//! ```rust
//! use llama::*;
//! fn main() -> Result<(), Error> {
//!     let ctx = Context::new()?;
//!     let mut module_ = Module::new(&ctx, "my_module")?;
//!     let builder = Builder::new(&ctx)?;
//!
//!     let i32 = Type::i32(&ctx)?;
//!
//!     let example_t = FuncType::new(i32, &[i32, i32])?;
//!     module_.declare_function(&builder, "example", example_t, |f| {
//!         let params = f.params();
//!         let r = builder.add(params[0], params[1], "add")?;
//!         builder.ret(r)
//!     })?;
//!
//!     println!("{}", module_);
//!     Ok(())
//! }
//! ```
//!

#[allow(non_snake_case)]
macro_rules! cstr {
    ($x:expr) => {
        std::ffi::CString::new($x).expect("Invalid C string")
    };
}

macro_rules! llvm_inner_impl {
    ($t:ty, $u:ty) => {
        impl<'a> LLVM<$u> for $t {
            fn llvm(&self) -> *mut $u {
                self.0.as_ptr()
            }
        }
    };
}

macro_rules! const_func {
    ($x:ident($(& $amp:ident$(,)?)? $($n:ident : $t:ty),*$(,)?) $b:block) => {
        pub fn $x<'b>($($amp,)? $($n : $t),*) -> Result<Const<'b>, Error> {
            unsafe {
                Ok(Const(Value::from_inner($b)?))
            }
        }
    }
}

macro_rules! instr {
    ($x:ident($(&$amp:ident$(,)?)? $($n:ident : $t:ty),*$(,)?) $b:block) => {
        pub fn $x<'b>($(& $amp,)? $($n : $t),*) -> Result<Instr<'b>, Error> {
            unsafe {
                Instr::from_inner($b)
            }
        }
    };

    ($ret:ident: $x:ident($(&$amp:ident$(,)?)? $($n:ident : $t:ty),*$(,)?) $b:block) => {
        pub fn $x<'b>($(& $amp,)? $($n : $t),*) -> Result<$ret<'b>, Error> {
            unsafe {
                Ok($ret::from_instr(Instr::from_inner($b)?))
            }
        }
    }
}

extern "C" {
    fn strlen(_: *const std::os::raw::c_char) -> usize;
}

mod attribute;
mod basic_block;
mod binary;
mod builder;
mod codegen;
mod r#const;
mod context;
mod error;
mod execution_engine;
mod instr;
mod metadata;
mod module;
mod pass_manager;
mod target;
mod r#type;
mod value;

pub(crate) use std::ffi::c_void;
pub(crate) use std::marker::PhantomData;
pub(crate) use std::os::raw::{c_char, c_int, c_uint};
pub(crate) use std::ptr::NonNull;

/// Re-export `llvm_sys` to provide access to any missing functionality
pub use llvm_sys as llvm;

pub use crate::attribute::Attribute;
pub use crate::basic_block::BasicBlock;
pub use crate::binary::Binary;
pub use crate::builder::Builder;
pub use crate::codegen::Codegen;
pub use crate::context::Context;
pub use crate::error::Error;
pub use crate::execution_engine::ExecutionEngine;
pub use crate::instr::*;
pub use crate::metadata::Metadata;
pub use crate::module::Module;
pub use crate::pass_manager::{
    transforms, FuncPassManager, ModulePassManager, PassManager, Transform,
};
pub use crate::r#const::Const;
pub use crate::r#type::{FuncType, StructType, Type, TypeKind};
pub use crate::target::{Target, TargetData, TargetMachine};
pub use crate::value::{AttributeIndex, Func, Value, ValueKind};

pub use llvm::{
    object::LLVMBinaryType as BinaryType,
    target::LLVMByteOrdering as ByteOrder,
    target_machine::{
        LLVMCodeGenOptLevel as CodeGenOptLevel, LLVMCodeModel as CodeModel,
        LLVMRelocMode as RelocMode,
    },
    LLVMAtomicOrdering as AtomicOrdering, LLVMAtomicRMWBinOp as AtomicRMWBinOp,
    LLVMCallConv as CallConv, LLVMDiagnosticSeverity as DiagnosticSeverity,
    LLVMInlineAsmDialect as InlineAsmDialect, LLVMIntPredicate as Icmp, LLVMLinkage as Linkage,
    LLVMModuleFlagBehavior as ModuleFlagBehavior, LLVMOpcode as OpCode, LLVMRealPredicate as Fcmp,
    LLVMThreadLocalMode as ThreadLocalMode, LLVMUnnamedAddr as UnnamedAddr,
    LLVMVisibility as Visibility,
};

/// Allows for llama types to be converted into LLVM pointers
pub trait LLVM<T> {
    /// Return a LLVM pointer
    fn llvm(&self) -> *mut T;
}

pub(crate) fn wrap_inner<T>(x: *mut T) -> Result<NonNull<T>, Error> {
    match NonNull::new(x) {
        Some(x) => Ok(x),
        None => Err(Error::NullPointer),
    }
}

/// Wraps LLVM messages, these are strings that should be freed using LLVMDisposeMessage
pub struct Message(*mut c_char);
impl Message {
    pub(crate) fn from_raw(c: *mut c_char) -> Message {
        Message(c)
    }

    /// Message length
    pub fn len(&self) -> usize {
        if self.0.is_null() {
            return 0;
        }

        unsafe { strlen(self.0) }
    }

    /// Returns true when the message is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

impl From<Message> for String {
    fn from(m: Message) -> String {
        m.as_ref().into()
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

/// Memory buffer wraps LLVMMemoryBufferRef
pub struct MemoryBuffer(NonNull<llvm::LLVMMemoryBuffer>);

llvm_inner_impl!(MemoryBuffer, llvm::LLVMMemoryBuffer);

impl MemoryBuffer {
    pub(crate) fn from_raw(ptr: *mut llvm::LLVMMemoryBuffer) -> Result<Self, Error> {
        Ok(MemoryBuffer(wrap_inner(ptr)?))
    }

    /// Create new memory buffer from file
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
            ) == 0
        };

        let message = Message::from_raw(message);
        if !ok {
            return Err(Error::Message(message));
        }

        Self::from_raw(mem)
    }

    /// Create new memory buffer from slice
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

    /// Number of bytes in buffer
    pub fn len(&self) -> usize {
        unsafe { llvm::core::LLVMGetBufferSize(self.0.as_ptr()) }
    }

    /// Returns true when the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write buffer to the specified file
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let mut f = std::fs::File::create(path)?;
        std::io::Write::write_all(&mut f, self.as_ref())?;
        Ok(())
    }
}

impl AsRef<[u8]> for MemoryBuffer {
    fn as_ref(&self) -> &[u8] {
        let size = self.len();
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

/// Load a shared library
pub fn load_library(filename: impl AsRef<std::path::Path>) -> bool {
    let filename = cstr!(filename
        .as_ref()
        .to_str()
        .expect("Invalid filename in call to load_library"));
    unsafe { llvm::support::LLVMLoadLibraryPermanently(filename.as_ptr()) == 0 }
}

/// Add a symbol
pub fn add_symbol<T>(filename: impl AsRef<str>, x: &mut T) {
    let filename = cstr!(filename.as_ref());
    unsafe { llvm::support::LLVMAddSymbol(filename.as_ptr(), x as *mut T as *mut c_void) }
}

/// Get the default target triple
pub fn default_target_triple() -> &'static str {
    unsafe {
        let ptr = llvm::target_machine::LLVMGetDefaultTargetTriple();
        let slice = std::slice::from_raw_parts(ptr as *const u8, strlen(ptr));
        std::str::from_utf8_unchecked(slice)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn codegen() -> Result<(), Error> {
        let context = Context::new()?;
        let mut module = Module::new(&context, "test_codegen")?;

        let builder = Builder::new(&context)?;

        let i32 = Type::int(&context, 32)?;

        let ft = FuncType::new(i32, &[i32, i32])?;
        module.declare_function(&builder, "testing", ft, |f| {
            let params = f.params();
            let a = builder.add(params[0], params[1], "a")?;
            builder.ret(&a)
        })?;

        println!("{}", module);

        let engine = ExecutionEngine::new_mcjit(&module, 2)?;

        let testing: extern "C" fn(i32, i32) -> i32 = unsafe { engine.function("testing")? };

        let x: i32 = testing(1i32, 2i32);
        assert_eq!(x, 3);

        Codegen::new(&module, &["testing"], true)?;

        Ok(())
    }

    #[test]
    fn if_then_else() -> Result<(), Error> {
        let ctx = Context::new()?;
        let mut module = Module::new(&ctx, "test_if_then_else")?;
        let builder = Builder::new(&ctx)?;

        let f32 = Type::float(&ctx)?;
        let ft = FuncType::new(f32, &[f32])?;
        module.declare_function(&builder, "testing", ft, |f| {
            let params = f.params();
            let cond = builder.fcmp(
                Fcmp::LLVMRealULT,
                &params[0],
                Const::real(f32, 10.0)?,
                "cond",
            )?;
            let a = Const::real(f32, 1.0)?;
            let b = Const::real(f32, 2.0)?;
            let ite = builder.if_then_else(cond, |_| Ok(a), |_| Ok(b))?;
            builder.ret(ite)
        })?;

        println!("{}", module);

        {
            let engine = ExecutionEngine::new(&module)?;
            let testing: extern "C" fn(f32) -> f32 = unsafe { engine.function("testing")? };
            let x = testing(11.0);
            let y = testing(9.0);

            assert_eq!(x, 2.0);
            assert_eq!(y, 1.0);
        }

        Codegen::new(&module, &["testing"], false)?;

        Ok(())
    }

    #[test]
    fn for_loop() -> Result<(), Error> {
        let ctx = Context::new()?;
        let mut module = Module::new(&ctx, "test_for_loop")?;
        let build = Builder::new(&ctx)?;

        let i64 = Type::int(&ctx, 64)?;

        let ft = FuncType::new(i64, &[i64])?;
        module.declare_function(&build, "testing", ft, |f| {
            let params = f.params();
            let one = Const::int_sext(i64, 1)?;
            let f = build.for_loop(
                Const::int_sext(&i64, 0)?,
                |x| build.icmp(Icmp::LLVMIntSLT, x, params[0], "cond"),
                |x| build.add(x, one, "add"),
                |x| Ok(*x),
            )?;
            build.ret(f)
        })?;

        println!("{}", module);

        {
            let engine = ExecutionEngine::new(&module)?;
            let testing: extern "C" fn(i64) -> i64 = unsafe { engine.function("testing")? };
            let x = testing(10);

            println!("{}", x);
            assert_eq!(x, 9);

            let x = testing(100);
            assert_eq!(x, 99);
        }

        Codegen::new(&module, &["testing"], true)?;

        Ok(())
    }
}
