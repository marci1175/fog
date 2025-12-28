use std::{rc::Rc, sync::Arc};

use common::{compiler::ProjectConfig, ty::OrdSet};
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

    c.bench_function("Parse big sourec file", |b| {
        b.iter(|| {
            parser
                .parse(Rc::new(common::dashmap::DashMap::new()))
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
