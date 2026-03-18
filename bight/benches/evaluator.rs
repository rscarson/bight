#![feature(test)]

extern crate test;

use std::hint::black_box;

use bight::{
    evaluator::{EvaluatorTable, SourceTable},
    sync::{Rc, RcStr},
    table::TableMut,
};
use test::*;

#[bench]
fn one_string(b: &mut Bencher) {
    let mut source = SourceTable::default();
    let hi: Rc<str> = black_box("hi").into();
    source.set((0, 0).into(), Some(hi.clone()));
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.set_source((0, 0), Some(hi.clone()));
        e.evaluate();
    });
}

#[bench]
fn one_formula(b: &mut Bencher) {
    let mut source = SourceTable::default();
    let formula: Rc<str> = black_box("=1 + 1").into();
    source.set((0, 0).into(), Some(formula.clone()));
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.set_source((0, 0), Some(formula.clone()));
        e.evaluate();
    });
}

#[bench]
fn strings_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    for i in 0..1000 {
        source.set((i, i).into(), Some(black_box(i.to_string().into())));
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn same_string_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    let s: RcStr = black_box("hi!".into());
    for i in 0..1000 {
        source.set((i, i).into(), Some(s.clone()));
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn independent_formulas_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    for i in 0..1000 {
        source.set((i, i).into(), Some(black_box(format!("={i} * {i}").into())));
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn same_formula_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    let formula: RcStr = black_box("=1 + 1".into());
    for i in 0..1000 {
        source.set((i, i).into(), Some(formula.clone()));
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn chained_formulas_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    source.set((0, 0).into(), Some(black_box("=1".into())));
    for i in 0..(1000 - 1) {
        source.set(
            (i + 1, i + 1).into(),
            Some(black_box(format!("=GET({i}, {i}) + 1",).into())),
        );
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn dep_cycle_formulas_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    source.set((0, 0).into(), Some(black_box("=GET(1000, 1000)".into())));
    for i in 0..(1000 - 1) {
        source.set(
            (i + 1, i + 1).into(),
            Some(black_box(format!("=GET({i}, {i}) + 1",).into())),
        );
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}

#[bench]
fn big_coord_formulas_large(b: &mut Bencher) {
    let mut source = SourceTable::default();
    source.set((100000, 100000).into(), Some(black_box("=1".into())));
    for i in 100000..(100000 + 1000 - 1) {
        source.set(
            (i + 1, i + 1).into(),
            Some(black_box(format!("=GET({i}, {i}) + 1",).into())),
        );
    }
    let mut e = EvaluatorTable::new(source.clone());
    b.iter(|| {
        e.invalidate_all_cells();
        e.evaluate();
    });
}
