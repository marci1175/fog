use inkwell::basic_block::BasicBlock;

pub mod codegen;
pub mod error;

/// Serves as a way to store information about the current loop body we are currently in.
#[derive(Debug, Clone)]
pub struct LoopBodyBlocks<'ctx> {
    /// The BasicBlock of the loop's body
    pub loop_body: BasicBlock<'ctx>,

    /// The BasicBlock of the code's continuation. This gets executed when we break out of the `loop_body`.
    pub loop_body_exit: BasicBlock<'ctx>,
}

impl<'ctx> LoopBodyBlocks<'ctx> {
    pub fn new(loop_body: BasicBlock<'ctx>, loop_body_exit: BasicBlock<'ctx>) -> Self {
        Self {
            loop_body,
            loop_body_exit,
        }
    }
}
