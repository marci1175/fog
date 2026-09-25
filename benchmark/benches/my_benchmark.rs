use std::rc::Rc;

use codegen::irgen::start_codegen;
use common::
    codegen::ty::OrdSet
;
use criterion::{Criterion, criterion_group, criterion_main};
use parser::{parser::Settings, tokenizer::tokenize};

fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
