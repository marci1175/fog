use std::{path::PathBuf, rc::Rc, sync::Arc};

use codegen::{import::import_user_lib_functions, irgen::generate_ir, llvm_codegen};
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
use parser::{parser_instance::Parser, tokenizer::tokenize};

fn criterion_benchmark(c: &mut Criterion)
{
    let (tokens, dbg_i, _) = tokenize(include_str!("benchmark_input.f"), None).unwrap();

    // Create Parser instance
    let mut parser = Parser::new(
        tokens,
        dbg_i,
        ProjectConfig::default(),
        vec![],
        OrdSet::new(),
    );

    unsafe {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();
    }

    let ctx = Context::create();
    let builder = ctx.create_builder();
    let module = ctx.create_module("main");

    c.bench_function("Parse source file", |b| {
        b.iter(|| {
            parser
                .parse(Rc::new(common::dashmap::DashMap::new()))
                .unwrap();

            import_user_lib_functions(
                &ctx,
                &module,
                Rc::new(parser.imported_functions().clone()),
                Rc::new(parser.function_table().clone()),
                parser.custom_types(),
            )
            .unwrap();

            generate_ir(
                Rc::new(parser.function_table().clone()),
                &ctx,
                &module,
                &builder,
                parser.custom_types(),
                true,
                "",
                "benchmark_codegen_no_path",
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
