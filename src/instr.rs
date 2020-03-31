use crate::*;

/// Instruction value
#[derive(Copy)]
pub struct Instr<'a>(pub(crate) Value<'a>);

macro_rules! instr_type {
    ($(#[$meta:meta])* $name:ident) => {
        #[derive(Clone, Copy)]
        $(#[$meta])*
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
            /// Convert from `Instr`
            pub fn from_instr(i: Instr<'a>) -> Self {
                $name(i.into())
            }

            /// Convert to `Instr`
            pub fn to_instr(self) -> Instr<'a> {
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

instr_type!(
    #[doc = "GEP instruction"]
    InstrGep
);
instr_type!(
    #[doc = "Alloca instruction"]
    InstrAlloca
);
instr_type!(
    #[doc = "Icmp instruction"]
    InstrIcmp
);
instr_type!(
    #[doc = "Fcmp instruction"]
    InstrFcmp
);
instr_type!(
    #[doc = "Phi instruction"]
    InstrPhi
);
instr_type!(
    #[doc = "Call instruction"]
    InstrCall
);
instr_type!(
    #[doc = "Switch instruction"]
    InstrSwitch
);
instr_type!(
    #[doc = "IndirectBr instruction"]
    InstrIndirectBr
);

impl<'a> Instr<'a> {
    /// Wrap an LLVMValue as `Instr`
    pub fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Self, Error> {
        Ok(Instr(Value::from_inner(ptr)?))
    }

    /// Get the parent block
    pub fn parent(self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetInstructionParent(self.as_ref().llvm()))
        }
    }

    /// Get the next instruction
    pub fn next_instruction(self) -> Result<Instr<'a>, Error> {
        unsafe {
            Ok(Instr(Value::from_inner(
                llvm::core::LLVMGetNextInstruction(self.as_ref().llvm()),
            )?))
        }
    }

    /// Get the previous instruction
    pub fn prev_instruction(self) -> Result<Instr<'a>, Error> {
        unsafe {
            Ok(Instr(Value::from_inner(
                llvm::core::LLVMGetPreviousInstruction(self.as_ref().llvm()),
            )?))
        }
    }

    /// Remove and erase an instruction from its parent
    pub fn delete(self) {
        unsafe { llvm::core::LLVMInstructionEraseFromParent(self.as_ref().llvm()) }
    }

    /// Remove an instruction from its parent
    pub fn remove_from_parent(self) {
        unsafe { llvm::core::LLVMInstructionRemoveFromParent(self.as_ref().llvm()) }
    }

    /// Get an instruction's OpCode
    pub fn op_code(self) -> OpCode {
        unsafe { llvm::core::LLVMGetInstructionOpcode(self.as_ref().llvm()) }
    }

    /// Returns true when an instruction has associated metadata
    pub fn has_metadata(self) -> bool {
        unsafe { llvm::core::LLVMHasMetadata(self.as_ref().llvm()) == 1 }
    }

    /// Get associated metadata
    pub fn get_metadata(self, id: u32) -> Result<Metadata<'a>, Error> {
        unsafe {
            Ok(Metadata(Value::from_inner(llvm::core::LLVMGetMetadata(
                self.as_ref().llvm(),
                id,
            ))?))
        }
    }

    /// Set associated metadata
    pub fn set_metadata(self, id: u32, meta: Metadata<'a>) {
        unsafe { llvm::core::LLVMSetMetadata(self.as_ref().llvm(), id, meta.as_ref().llvm()) }
    }

    /// Get number of successors
    pub fn num_successors(self) -> usize {
        unsafe { llvm::core::LLVMGetNumSuccessors(self.as_ref().llvm()) as usize }
    }

    /// Get a successor by index
    pub fn successor(self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    /// Set a successor block
    pub fn set_successor(&mut self, index: usize, bb: BasicBlock<'a>) {
        unsafe { llvm::core::LLVMSetSuccessor(self.as_ref().llvm(), index as c_uint, bb.llvm()) }
    }

    /// Returns true if the instruction is conditional
    pub fn is_conditional(self) -> bool {
        unsafe { llvm::core::LLVMIsConditional(self.as_ref().llvm()) == 1 }
    }

    /// Get the condition of a conditional
    pub fn condition(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCondition(self.as_ref().llvm())) }
    }

    /// Set the condition of a conditional
    pub fn set_condition(&mut self, cond: impl AsRef<Value<'a>>) {
        unsafe { llvm::core::LLVMSetCondition(self.as_ref().llvm(), cond.as_ref().llvm()) }
    }
}

impl<'a> InstrIcmp<'a> {
    /// Get integer comparison predicate
    pub fn predicate(self) -> Icmp {
        unsafe { llvm::core::LLVMGetICmpPredicate(self.as_ref().llvm()) }
    }
}

impl<'a> InstrFcmp<'a> {
    /// Get float comparison predicate
    pub fn predicate(self) -> Fcmp {
        unsafe { llvm::core::LLVMGetFCmpPredicate(self.as_ref().llvm()) }
    }
}

impl<'a> InstrAlloca<'a> {
    /// Get alloca type
    pub fn get_type(self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetAllocatedType(self.as_ref().llvm())) }
    }
}

impl<'a> InstrPhi<'a> {
    /// Add incoming items
    pub fn add_incoming(&mut self, items: impl AsRef<[(Value<'a>, BasicBlock<'a>)]>) {
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

    /// Return the number of incoming blocks/values
    pub fn count_incoming(self) -> usize {
        unsafe { llvm::core::LLVMCountIncoming(self.as_ref().llvm()) as usize }
    }

    /// Get incoming block
    pub fn incoming_value(self, index: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetIncomingValue(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    /// Get incoming value
    pub fn incoming_block(self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetIncomingBlock(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }
}

impl<'a> InstrGep<'a> {
    /// Returns true when the GEP instruction is marked as in-bounds
    pub fn in_bounds(self) -> bool {
        unsafe { llvm::core::LLVMIsInBounds(self.as_ref().llvm()) == 1 }
    }

    /// Mark the GEP instruction as in-bounds
    pub fn set_in_bounds(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetIsInBounds(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }
}

impl<'a> InstrCall<'a> {
    /// Get number of operands for a call instruction
    pub fn num_operands(self) -> usize {
        unsafe { llvm::core::LLVMGetNumArgOperands(self.as_ref().llvm()) as usize }
    }

    /// Get called function type
    pub fn function_type(self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetCalledFunctionType(self.as_ref().llvm())) }
    }

    /// Get called value
    pub fn value(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCalledValue(self.as_ref().llvm())) }
    }

    /// Returns true when the call is marked as a tail call
    pub fn is_tail_call(self) -> bool {
        unsafe { llvm::core::LLVMIsTailCall(self.as_ref().llvm()) == 1 }
    }

    /// Mark the call as a tail call
    pub fn set_tail_call(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetTailCall(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }
}

impl<'a> InstrSwitch<'a> {
    /// Get the default destination block
    pub fn default_dest(self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSwitchDefaultDest(self.as_ref().llvm()))
        }
    }

    /// Add a new case
    pub fn add_case(&mut self, on_val: impl AsRef<Value<'a>>, dest: BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMAddCase(self.as_ref().llvm(), on_val.as_ref().llvm(), dest.llvm())
        }
    }
}

impl<'a> InstrIndirectBr<'a> {
    /// Add a destination
    pub fn add_dest(&mut self, dest: BasicBlock<'a>) {
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
