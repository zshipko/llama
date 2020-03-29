use crate::*;

#[derive(Copy)]
pub struct Instr<'a>(pub(crate) Value<'a>);

macro_rules! instr_type {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name<'a>(pub(crate) Value<'a>);
        impl<'a> AsRef<Value<'a>> for $name<'a> {
            fn as_ref(&self) -> &Value<'a> {
                &self.0
            }
        }

        impl<'a> From<$name<'a>> for Value<'a> {
            fn from(x: $name<'a>) -> Value<'a> {
                x.0
            }
        }

        impl<'a> $name<'a> {
            pub fn from_instr(i: Instr<'a>) -> Self {
                $name(i.into())
            }

            pub fn into_instr(self) -> Instr<'a> {
                Instr(self.0)
            }
        }
    };
}

impl<'a> AsRef<Value<'a>> for Instr<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Instr<'a>> for Value<'a> {
    fn from(x: Instr<'a>) -> Value<'a> {
        x.0
    }
}

instr_type!(InstrGep);
instr_type!(InstrAlloca);
instr_type!(InstrIcmp);
instr_type!(InstrFcmp);
instr_type!(InstrPhi);
instr_type!(InstrCall);
instr_type!(InstrSwitch);
instr_type!(InstrIndirectBr);

impl<'a> Instr<'a> {
    pub fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Self, Error> {
        Ok(Instr(Value::from_inner(ptr)?))
    }

    pub fn parent(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetInstructionParent(self.as_ref().llvm()))
        }
    }

    pub fn next_instruction(&self) -> Result<Instr<'a>, Error> {
        unsafe {
            Ok(Instr(Value::from_inner(
                llvm::core::LLVMGetNextInstruction(self.as_ref().llvm()),
            )?))
        }
    }

    pub fn prev_instruction(&self) -> Result<Instr<'a>, Error> {
        unsafe {
            Ok(Instr(Value::from_inner(
                llvm::core::LLVMGetPreviousInstruction(self.as_ref().llvm()),
            )?))
        }
    }

    pub fn delete(self) {
        unsafe { llvm::core::LLVMInstructionEraseFromParent(self.as_ref().llvm()) }
    }

    pub fn remove_from_parent(&self) {
        unsafe { llvm::core::LLVMInstructionRemoveFromParent(self.as_ref().llvm()) }
    }

    pub fn opcode(&self) -> OpCode {
        unsafe { llvm::core::LLVMGetInstructionOpcode(self.as_ref().llvm()) }
    }

    pub fn has_metadata(&self) -> bool {
        unsafe { llvm::core::LLVMHasMetadata(self.as_ref().llvm()) == 1 }
    }

    pub fn get_metadata(&self, id: u32) -> Result<Metadata<'a>, Error> {
        unsafe {
            Ok(Metadata(Value::from_inner(llvm::core::LLVMGetMetadata(
                self.as_ref().llvm(),
                id,
            ))?))
        }
    }

    pub fn set_metadata(&self, id: u32, meta: &Metadata<'a>) {
        unsafe { llvm::core::LLVMSetMetadata(self.as_ref().llvm(), id, meta.as_ref().llvm()) }
    }

    pub fn num_successors(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumSuccessors(self.as_ref().llvm()) as usize }
    }

    pub fn successor(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn set_successor(&mut self, index: usize, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMSetSuccessor(self.as_ref().llvm(), index as c_uint, bb.llvm()) }
    }

    pub fn is_conditional(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn condition(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCondition(self.as_ref().llvm())) }
    }

    pub fn set_condition(&mut self, cond: impl AsRef<Value<'a>>) {
        unsafe { llvm::core::LLVMSetCondition(self.as_ref().llvm(), cond.as_ref().llvm()) }
    }
}

impl<'a> InstrIcmp<'a> {
    pub fn predicate(&self) -> Icmp {
        unsafe { llvm::core::LLVMGetICmpPredicate(self.as_ref().llvm()) }
    }
}

impl<'a> InstrFcmp<'a> {
    pub fn predicate(&self) -> Fcmp {
        unsafe { llvm::core::LLVMGetFCmpPredicate(self.as_ref().llvm()) }
    }
}

impl<'a> InstrAlloca<'a> {
    pub fn get_type(&self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetAllocatedType(self.as_ref().llvm())) }
    }
}

impl<'a> InstrPhi<'a> {
    pub fn add_incoming<'b>(&'b self, items: impl AsRef<[(&'b Value<'a>, &'b BasicBlock<'a>)]>) {
        let mut values: Vec<*mut llvm::LLVMValue> =
            items.as_ref().iter().map(|(v, _)| v.llvm()).collect();

        let mut blocks: Vec<*mut llvm::LLVMBasicBlock> =
            items.as_ref().iter().map(|(_, v)| v.llvm()).collect();

        let count = values.len();
        unsafe {
            llvm::core::LLVMAddIncoming(
                self.as_ref().llvm(),
                values.as_mut_ptr(),
                blocks.as_mut_ptr(),
                count as c_uint,
            )
        }
    }

    pub fn count_incoming(&self) -> usize {
        unsafe { llvm::core::LLVMCountIncoming(self.as_ref().llvm()) as usize }
    }

    pub fn incoming_value(&self, index: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetIncomingValue(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn incoming_block(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetIncomingBlock(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }
}

impl<'a> InstrGep<'a> {
    pub fn in_bounds(&self) -> bool {
        unsafe { llvm::core::LLVMIsInBounds(self.as_ref().llvm()) == 1 }
    }

    pub fn set_in_bounds(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetIsInBounds(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }
}

impl<'a> InstrCall<'a> {
    pub fn num_operands(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumArgOperands(self.as_ref().llvm()) as usize }
    }

    pub fn function_type(&self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetCalledFunctionType(self.as_ref().llvm())) }
    }

    pub fn value(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCalledValue(self.as_ref().llvm())) }
    }

    pub fn is_tail_call(&self) -> bool {
        unsafe { llvm::core::LLVMIsTailCall(self.as_ref().llvm()) == 1 }
    }

    pub fn set_tail_call(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetTailCall(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }
}

impl<'a> InstrSwitch<'a> {
    pub fn default_dest(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSwitchDefaultDest(self.as_ref().llvm()))
        }
    }

    pub fn add_case(&self, on_val: impl AsRef<Value<'a>>, dest: &BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMAddCase(self.as_ref().llvm(), on_val.as_ref().llvm(), dest.llvm())
        }
    }
}

impl<'a> InstrIndirectBr<'a> {
    pub fn add_dest(&self, dest: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAddDestination(self.as_ref().llvm(), dest.llvm()) }
    }
}

impl<'a> Clone for Instr<'a> {
    fn clone(&self) -> Instr<'a> {
        unsafe {
            Instr(
                Value::from_inner(llvm::core::LLVMInstructionClone(self.as_ref().llvm())).unwrap(),
            )
        }
    }
}
