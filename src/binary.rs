use crate::*;

/// Binary is used to store compiled binary objects
pub struct Binary(NonNull<llvm::object::LLVMOpaqueBinary>);

llvm_inner_impl!(Binary, llvm::object::LLVMOpaqueBinary);

impl<'a> Drop for Binary {
    fn drop(&mut self) {
        unsafe { llvm::object::LLVMDisposeBinary(self.llvm()) }
    }
}

impl<'a> Binary {
    /// Create a new binary object
    pub fn new(ctx: &Context, data: &MemoryBuffer) -> Result<Binary, Error> {
        let mut message = std::ptr::null_mut();
        let bin = unsafe { llvm::object::LLVMCreateBinary(data.llvm(), ctx.llvm(), &mut message) };

        let message = Message::from_raw(message);

        match wrap_inner(bin) {
            Ok(bin) => Ok(Binary(bin)),
            Err(_) => Err(Error::Message(message)),
        }
    }

    /// Get binary file type
    pub fn get_type(&self) -> BinaryType {
        unsafe { llvm::object::LLVMBinaryGetType(self.llvm()) }
    }

    /// Write binary object to file
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let buffer = unsafe { llvm::object::LLVMBinaryCopyMemoryBuffer(self.llvm()) };
        let buf = MemoryBuffer::from_raw(buffer)?;
        buf.write_to_file(path)
    }
}

impl<'a> AsRef<[u8]> for Binary {
    fn as_ref(&self) -> &[u8] {
        let buffer = unsafe { llvm::object::LLVMBinaryCopyMemoryBuffer(self.llvm()) };
        let buf = MemoryBuffer::from_raw(buffer).unwrap();
        let ptr = buf.as_ref().as_ptr();
        let len = buf.len();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}
