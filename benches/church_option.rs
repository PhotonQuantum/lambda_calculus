#![cfg(feature = "encoding")]

#![feature(test)]
extern crate test;

#[macro_use]
extern crate lambda_calculus as lambda;

use test::Bencher;
use lambda::church::option::*;
use lambda::church::numerals::succ;
use lambda::*;

#[bench]
fn church_is_none(b: &mut Bencher) {
    b.iter(|| { beta(app!(is_none(), none()), HAP, 0) } );
}

#[bench]
fn church_is_some(b: &mut Bencher) {
    b.iter(|| { beta(app!(is_some(), none()), HAP, 0) } );
}

#[bench]
fn church_map_or(b: &mut Bencher) {
    b.iter(|| { beta(app!(map_or(), 0.into(), succ(), none()), HAP, 0) } );
}
