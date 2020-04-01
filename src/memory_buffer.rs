use crate::*;

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
