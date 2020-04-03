#![allow(clippy::should_implement_trait)]
#![deny(missing_docs)]

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
mod memory_buffer;
mod message;
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
pub use crate::memory_buffer::MemoryBuffer;
pub use crate::message::Message;
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

/// Load a shared library
pub fn load_library(filename: impl AsRef<std::path::Path>) -> bool {
    let filename = cstr!(filename
        .as_ref()
        .to_str()
        .expect("Invalid filename in call to load_library"));
    unsafe { llvm::support::LLVMLoadLibraryPermanently(filename.as_ptr()) == 0 }
}

/// Add a symbol
pub fn add_symbol<T>(name: impl AsRef<str>, x: *mut T) {
    let name = cstr!(name.as_ref());
    unsafe { llvm::support::LLVMAddSymbol(name.as_ptr(), x as *mut c_void) }
}

#[macro_export]
/// Add symbols to the global namespace
///
/// There are a few ways to use this macro:
///
/// 1) `symbol!(foo, bar)`: adds symbols for `foo` and `bar` in the local namespace, this
///    is particularly useful for adding symbols for C functions
/// 2) `symbol!(module.fn foo: function_type)`: adds a symbol `foo` to the global namespace and defines
///    a function in the specified `module`
/// 3) `symbol!(module.fn foo(i32) -> i32`: adds a symbol `foo` to the global namespace, creates
///    a function type using the specified Rust function declaration (using `Type`s instead of
///    Rust types) and defines a function in the specified `module`
macro_rules! symbol {
    ($($name:ident);*) => {
        $(
            $crate::add_symbol(stringify!($name), $name as *mut std::ffi::c_void);
        )*
    };
    ($module:ident.fn $name:ident($($x:ident),*) -> $ret:expr) => {{
        symbol!($name);
        let t = $crate::FuncType::new($ret, &[$($x),*])?;
        $module.define_function(stringify!($name), t)
    }};
    ($module:ident.fn $name:ident: $t:expr) => {{
        symbol!($name);
        $module.define_function(stringify!($name), $t)
    }};
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
mod tests;
