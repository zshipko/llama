use crate::*;

pub struct Instruction<'a>(pub(crate) Value<'a>);

impl<'a> AsRef<Value<'a>> for Instruction<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Instruction<'a>> for Value<'a> {
    fn from(x: Instruction<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Instruction<'a> {
    pub fn parent(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetInstructionParent(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn next_instruction(&self) -> Result<Instruction<'a>, Error> {
        unsafe {
            Ok(Instruction(Value::from_inner(
                llvm::core::LLVMGetNextInstruction(self.as_ref().llvm_inner()),
            )?))
        }
    }

    pub fn prev_instruction(&self) -> Result<Instruction<'a>, Error> {
        unsafe {
            Ok(Instruction(Value::from_inner(
                llvm::core::LLVMGetPreviousInstruction(self.as_ref().llvm_inner()),
            )?))
        }
    }

    pub fn delete(self) {
        unsafe { llvm::core::LLVMInstructionEraseFromParent(self.as_ref().llvm_inner()) }
    }

    pub fn remove_from_parent(&self) {
        unsafe { llvm::core::LLVMInstructionRemoveFromParent(self.as_ref().llvm_inner()) }
    }

    pub fn opcode(&self) -> OpCode {
        unsafe { llvm::core::LLVMGetInstructionOpcode(self.as_ref().llvm_inner()) }
    }

    pub fn icmp_predicate(&self) -> ICmp {
        unsafe { llvm::core::LLVMGetICmpPredicate(self.as_ref().llvm_inner()) }
    }

    pub fn fcmp_predicate(&self) -> FCmp {
        unsafe { llvm::core::LLVMGetFCmpPredicate(self.as_ref().llvm_inner()) }
    }

    pub fn alloca_type(&self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetAllocatedType(self.as_ref().llvm_inner())) }
    }

    pub fn phi_add_incoming<'b>(
        &'b self,
        items: impl AsRef<[(&'b Value<'a>, &'b BasicBlock<'a>)]>,
    ) {
        let mut values: Vec<*mut llvm::LLVMValue> =
            items.as_ref().iter().map(|(v, _)| v.llvm_inner()).collect();

        let mut blocks: Vec<*mut llvm::LLVMBasicBlock> =
            items.as_ref().iter().map(|(_, v)| v.llvm_inner()).collect();

        let count = values.len();
        unsafe {
            llvm::core::LLVMAddIncoming(
                self.as_ref().llvm_inner(),
                values.as_mut_ptr(),
                blocks.as_mut_ptr(),
                count as c_uint,
            )
        }
    }

    pub fn phi_count_incoming(&self) -> usize {
        unsafe { llvm::core::LLVMCountIncoming(self.as_ref().llvm_inner()) as usize }
    }

    pub fn phi_incoming_value(&self, index: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetIncomingValue(
                self.as_ref().llvm_inner(),
                index as c_uint,
            ))
        }
    }

    pub fn phi_incoming_block(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetIncomingBlock(
                self.as_ref().llvm_inner(),
                index as c_uint,
            ))
        }
    }

    pub fn gep_is_in_bounds(&self) -> bool {
        unsafe { llvm::core::LLVMIsInBounds(self.as_ref().llvm_inner()) == 1 }
    }

    pub fn set_gep_in_bounds(&self, b: bool) {
        unsafe { llvm::core::LLVMSetIsInBounds(self.as_ref().llvm_inner(), if b { 1 } else { 0 }) }
    }

    pub fn call_num_operands(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumArgOperands(self.as_ref().llvm_inner()) as usize }
    }

    pub fn call_function_type(&self) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::core::LLVMGetCalledFunctionType(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn call_value(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCalledValue(self.as_ref().llvm_inner())) }
    }

    pub fn is_tail_call(&self) -> bool {
        unsafe { llvm::core::LLVMIsTailCall(self.as_ref().llvm_inner()) == 1 }
    }

    pub fn set_tail_call(&self, b: bool) {
        unsafe { llvm::core::LLVMSetTailCall(self.as_ref().llvm_inner(), if b { 1 } else { 0 }) }
    }

    pub fn terminator_num_successors(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumSuccessors(self.as_ref().llvm_inner()) as usize }
    }

    pub fn terminator_successor(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm_inner(),
                index as c_uint,
            ))
        }
    }

    pub fn terminator_set_successor(&self, index: usize, bb: &BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMSetSuccessor(
                self.as_ref().llvm_inner(),
                index as c_uint,
                bb.llvm_inner(),
            )
        }
    }

    pub fn terminator_is_conditional(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm_inner(),
                index as c_uint,
            ))
        }
    }

    pub fn terminator_condition(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCondition(self.as_ref().llvm_inner())) }
    }

    pub fn terminator_set_condition(&self, cond: impl AsRef<Value<'a>>) {
        unsafe {
            llvm::core::LLVMSetCondition(self.as_ref().llvm_inner(), cond.as_ref().llvm_inner())
        }
    }

    pub fn switch_default_dest(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSwitchDefaultDest(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn switch_add_case(&self, on_val: impl AsRef<Value<'a>>, dest: &BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMAddCase(
                self.as_ref().llvm_inner(),
                on_val.as_ref().llvm_inner(),
                dest.llvm_inner(),
            )
        }
    }

    pub fn indirect_br_add_dest(&self, dest: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAddDestination(self.as_ref().llvm_inner(), dest.llvm_inner()) }
    }
}

impl<'a> Clone for Instruction<'a> {
    fn clone(&self) -> Instruction<'a> {
        unsafe {
            Instruction(
                Value::from_inner(llvm::core::LLVMInstructionClone(self.as_ref().llvm_inner()))
                    .unwrap(),
            )
        }
    }
}
