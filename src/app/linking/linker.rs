use std::{ffi::{CStr, CString}, path::PathBuf};

use inkwell::{llvm_sys::{core::LLVMDisposeMessage, target_machine::LLVMTargetMachineEmitToFile}, module::Module, targets::TargetMachine};

pub fn link_llvm_to_target<'ctx>(module: &Module<'ctx>, target: TargetMachine, path_to_output: PathBuf) -> anyhow::Result<()> {
    // lld_rx::link(lld_rx::LldFlavor::Coff, vec!["--version".to_string()]);

    panic!("linking is not supported right now.");
    
    Ok(())
}