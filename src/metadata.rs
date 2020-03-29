use crate::*;

#[derive(Clone, Copy)]
pub struct Metadata<'a>(pub(crate) Value<'a>);

impl<'a> AsRef<Value<'a>> for Metadata<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Metadata<'a>> for Value<'a> {
    fn from(x: Metadata<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Metadata<'a> {
    /// Create a string attribute
    pub fn new_string(ctx: &Context<'a>, k: impl AsRef<str>) -> Result<Metadata<'a>, Error> {
        let k = k.as_ref();
        unsafe {
            Ok(Metadata(Value::from_inner(
                llvm::core::LLVMMetadataAsValue(
                    ctx.llvm(),
                    llvm::core::LLVMMDStringInContext2(
                        ctx.llvm(),
                        k.as_ptr() as *const c_char,
                        k.len(),
                    ),
                ),
            )?))
        }
    }

    /// Create an enum attribute
    pub fn new_node(
        ctx: &Context<'a>,
        mds: impl AsRef<[Metadata<'a>]>,
    ) -> Result<Metadata<'a>, Error> {
        let mut ptr: Vec<*mut llvm::LLVMOpaqueMetadata> = mds
            .as_ref()
            .iter()
            .map(|x| unsafe { llvm::core::LLVMValueAsMetadata(x.as_ref().llvm()) })
            .collect();
        unsafe {
            Ok(Metadata(Value::from_inner(
                llvm::core::LLVMMetadataAsValue(
                    ctx.llvm(),
                    llvm::core::LLVMMDNodeInContext2(ctx.llvm(), ptr.as_mut_ptr(), ptr.len()),
                ),
            )?))
        }
    }

    pub fn as_str(self) -> Result<&'a str, Error> {
        unsafe {
            let mut len = 0;
            let ptr = llvm::core::LLVMGetMDString(self.as_ref().llvm(), &mut len);
            if ptr.is_null() {
                return Err(Error::NullPointer);
            }

            let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            let s = std::str::from_utf8(slice)?;
            Ok(s)
        }
    }

    pub fn node(self) -> Vec<Metadata<'a>> {
        unsafe {
            let len = llvm::core::LLVMGetMDNodeNumOperands(self.as_ref().llvm());
            let mut a = vec![std::ptr::null_mut(); len as usize];
            llvm::core::LLVMGetMDNodeOperands(self.as_ref().llvm(), a.as_mut_ptr());
            a.into_iter()
                .map(|x| Metadata(Value::from_inner(x).unwrap()))
                .collect()
        }
    }
}
