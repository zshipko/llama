use crate::*;

pub struct Module<'a>(NonNull<llvm::LLVMModule>, PhantomData<&'a ()>);

impl<'a> LLVMInner<llvm::LLVMModule> for Module<'a> {
    fn llvm_inner(&self) -> *mut llvm::LLVMModule {
        self.0.as_ptr()
    }
}

impl<'a> Drop for Module<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeModule(self.llvm_inner()) }
    }
}

impl<'a> Module<'a> {
    pub fn new(ctx: &'a Context, name: impl AsRef<str>) -> Result<Module<'a>, Error> {
        let name = cstr!(name.as_ref());
        let m = unsafe {
            wrap_inner(llvm::core::LLVMModuleCreateWithNameInContext(
                name.as_ptr(),
                ctx.llvm_inner(),
            ))?
        };
        Ok(Module(m, PhantomData))
    }

    pub fn parse_bitcode(ctx: &Context, mem_buf: &MemoryBuffer) -> Option<Module<'a>> {
        let mut module = std::ptr::null_mut();
        let ok = unsafe {
            llvm::bit_reader::LLVMParseBitcodeInContext2(
                ctx.llvm_inner(),
                mem_buf.llvm_inner(),
                &mut module,
            ) == 1
        };

        if !ok {
            return None;
        }

        let module = match wrap_inner(module) {
            Ok(m) => m,
            Err(_) => return None,
        };

        Some(Module(module, PhantomData))
    }

    pub fn identifier(&self) -> Result<&str, Error> {
        let mut size = 0usize;
        unsafe {
            let s = llvm::core::LLVMGetModuleIdentifier(self.llvm_inner(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    pub fn set_target(&self, target: impl AsRef<str>) {
        let target = cstr!(target.as_ref());
        unsafe { llvm::core::LLVMSetTarget(self.llvm_inner(), target.as_ptr()) }
    }

    pub fn get_target(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetTarget(self.llvm_inner());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    pub fn set_data_layout(&self, layout: impl AsRef<str>) {
        let layout = cstr!(layout.as_ref());
        unsafe { llvm::core::LLVMSetDataLayout(self.llvm_inner(), layout.as_ptr()) }
    }

    pub fn get_data_layout(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetDataLayoutStr(self.llvm_inner());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    pub fn write_bitcode_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<bool, Error> {
        let path = match path.as_ref().to_str() {
            Some(p) => cstr!(p),
            None => return Err(Error::InvalidPath),
        };

        let r = unsafe {
            llvm::bit_writer::LLVMWriteBitcodeToFile(self.llvm_inner(), path.as_ptr()) == 0
        };

        Ok(r)
    }

    pub fn write_bitcode_to_memory_buffer(&self) -> Result<MemoryBuffer, Error> {
        let mem = unsafe { llvm::bit_writer::LLVMWriteBitcodeToMemoryBuffer(self.llvm_inner()) };
        MemoryBuffer::from_raw(mem)
    }
}

impl<'a> std::fmt::Display for Module<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message =
            unsafe { Message::from_raw(llvm::core::LLVMPrintModuleToString(self.llvm_inner())) };
        write!(fmt, "{}", message.as_ref())
    }
}
