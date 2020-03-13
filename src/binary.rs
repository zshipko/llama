use crate::*;

pub struct Binary<'a>(NonNull<llvm::object::LLVMOpaqueBinary>, PhantomData<&'a ()>);

llvm_inner_impl!(Binary<'a>, llvm::object::LLVMOpaqueBinary);

impl<'a> Drop for Binary<'a> {
    fn drop(&mut self) {
        unsafe { llvm::object::LLVMDisposeBinary(self.llvm_inner()) }
    }
}

impl<'a> Binary<'a> {
    pub fn new(ctx: &Context, data: &MemoryBuffer) -> Result<Binary<'a>, Error> {
        let mut message = std::ptr::null_mut();
        let bin = unsafe {
            llvm::object::LLVMCreateBinary(data.llvm_inner(), ctx.llvm_inner(), &mut message)
        };

        let message = Message::from_raw(message);

        match wrap_inner(bin) {
            Ok(bin) => Ok(Binary(bin, PhantomData)),
            Err(_) => Err(Error::Message(message)),
        }
    }

    pub fn get_type(&self) -> BinaryType {
        unsafe { llvm::object::LLVMBinaryGetType(self.llvm_inner()) }
    }

    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let buffer = unsafe { llvm::object::LLVMBinaryCopyMemoryBuffer(self.llvm_inner()) };
        let buf = MemoryBuffer::from_raw(buffer)?;
        buf.write_to_file(path)
    }
}

impl<'a> AsRef<[u8]> for Binary<'a> {
    fn as_ref(&self) -> &[u8] {
        let buffer = unsafe { llvm::object::LLVMBinaryCopyMemoryBuffer(self.llvm_inner()) };
        let buf = MemoryBuffer::from_raw(buffer).unwrap();
        let ptr = buf.as_ref().as_ptr();
        let len = buf.len();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}
