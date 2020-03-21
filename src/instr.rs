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
    pub fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Self, Error> {
        Ok(Instruction(Value::from_inner(ptr)?))
    }

    pub fn parent(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetInstructionParent(
                self.as_ref().llvm(),
            ))
        }
    }

    pub fn next_instruction(&self) -> Result<Instruction<'a>, Error> {
        unsafe {
            Ok(Instruction(Value::from_inner(
                llvm::core::LLVMGetNextInstruction(self.as_ref().llvm()),
            )?))
        }
    }

    pub fn prev_instruction(&self) -> Result<Instruction<'a>, Error> {
        unsafe {
            Ok(Instruction(Value::from_inner(
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

    pub fn icmp_predicate(&self) -> ICmp {
        unsafe { llvm::core::LLVMGetICmpPredicate(self.as_ref().llvm()) }
    }

    pub fn fcmp_predicate(&self) -> FCmp {
        unsafe { llvm::core::LLVMGetFCmpPredicate(self.as_ref().llvm()) }
    }

    pub fn alloca_type(&self) -> Result<Type<'a>, Error> {
        unsafe { Type::from_inner(llvm::core::LLVMGetAllocatedType(self.as_ref().llvm())) }
    }

    pub fn phi_add_incoming<'b>(
        &'b self,
        items: impl AsRef<[(&'b Value<'a>, &'b BasicBlock<'a>)]>,
    ) {
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

    pub fn phi_count_incoming(&self) -> usize {
        unsafe { llvm::core::LLVMCountIncoming(self.as_ref().llvm()) as usize }
    }

    pub fn phi_incoming_value(&self, index: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetIncomingValue(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn phi_incoming_block(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetIncomingBlock(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn gep_is_in_bounds(&self) -> bool {
        unsafe { llvm::core::LLVMIsInBounds(self.as_ref().llvm()) == 1 }
    }

    pub fn set_gep_in_bounds(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetIsInBounds(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }

    pub fn call_num_operands(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumArgOperands(self.as_ref().llvm()) as usize }
    }

    pub fn call_function_type(&self) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::core::LLVMGetCalledFunctionType(
                self.as_ref().llvm(),
            ))
        }
    }

    pub fn call_value(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCalledValue(self.as_ref().llvm())) }
    }

    pub fn is_tail_call(&self) -> bool {
        unsafe { llvm::core::LLVMIsTailCall(self.as_ref().llvm()) == 1 }
    }

    pub fn set_tail_call(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetTailCall(self.as_ref().llvm(), if b { 1 } else { 0 }) }
    }

    pub fn terminator_num_successors(&self) -> usize {
        unsafe { llvm::core::LLVMGetNumSuccessors(self.as_ref().llvm()) as usize }
    }

    pub fn terminator_successor(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn terminator_set_successor(&mut self, index: usize, bb: &BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMSetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
                bb.llvm(),
            )
        }
    }

    pub fn terminator_is_conditional(&self, index: usize) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSuccessor(
                self.as_ref().llvm(),
                index as c_uint,
            ))
        }
    }

    pub fn terminator_condition(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetCondition(self.as_ref().llvm())) }
    }

    pub fn terminator_set_condition(&mut self, cond: impl AsRef<Value<'a>>) {
        unsafe {
            llvm::core::LLVMSetCondition(self.as_ref().llvm(), cond.as_ref().llvm())
        }
    }

    pub fn switch_default_dest(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            BasicBlock::from_inner(llvm::core::LLVMGetSwitchDefaultDest(
                self.as_ref().llvm(),
            ))
        }
    }

    pub fn switch_add_case(&self, on_val: impl AsRef<Value<'a>>, dest: &BasicBlock<'a>) {
        unsafe {
            llvm::core::LLVMAddCase(
                self.as_ref().llvm(),
                on_val.as_ref().llvm(),
                dest.llvm(),
            )
        }
    }

    pub fn indirect_br_add_dest(&self, dest: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAddDestination(self.as_ref().llvm(), dest.llvm()) }
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
        unsafe {
            llvm::core::LLVMSetMetadata(self.as_ref().llvm(), id, meta.as_ref().llvm())
        }
    }
}

impl<'a> Clone for Instruction<'a> {
    fn clone(&self) -> Instruction<'a> {
        unsafe {
            Instruction(
                Value::from_inner(llvm::core::LLVMInstructionClone(self.as_ref().llvm()))
                    .unwrap(),
            )
        }
    }
}
