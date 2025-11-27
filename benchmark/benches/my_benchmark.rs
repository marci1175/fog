use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use fog_common::{compiler::ProjectConfig, ty::OrdSet};
use fog_parser::{parser_instance::Parser, tokenizer::tokenize};

fn criterion_benchmark(c: &mut Criterion) {
    let (tokens, dbg_i, _) = tokenize(include_str!("benchmark_input.f"), None).unwrap();

    // Create Parser instance
    let mut parser = Parser::new(tokens, dbg_i, ProjectConfig::default(), vec![], OrdSet::new());

    c.bench_function("Parse big sourec file", |b| b.iter(|| {
        parser.parse(fog_common::indexmap::IndexMap::new()).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);