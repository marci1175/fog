use std::rc::Rc;

use codegen::{import::import_user_lib_functions, irgen::generate_ir};
use common::{
    compiler::ProjectConfig,
    inkwell::{
        context::Context,
        llvm_sys::target::{
            LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters,
            LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
        },
    },
    ty::OrdSet,
};
use criterion::{Criterion, criterion_group, criterion_main};
use parser::{parser::ParserSettings, tokenizer::tokenize};

fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
