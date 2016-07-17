#![feature(test)]

extern crate test;
extern crate msgp;

use test::Bencher;
use msgp::{encode};

// Last result:

#[bench]
fn b_encode(b: &mut Bencher) {
    b.iter(|| {
        encode(&vec![0xffu8; 16385]);
    });
}
