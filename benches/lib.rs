#![feature(test)]

extern crate test;
extern crate msgp;

use test::Bencher;
use msgp::{encode, encode_slice};

// Last result:

#[bench]
fn b_encode(b: &mut Bencher) {
    b.iter(|| {
        encode(&mut vec![0xffu8; 16385]);
    });
}

#[bench]
fn b_encode_slice(b: &mut Bencher) {
    b.iter(|| {
        encode_slice((vec![0xffu8; 16385]).as_slice());
    });
}

#[bench]
fn b_encode_slice_2(b: &mut Bencher) {
    let buf = vec![0xffu8; 16385];
    b.iter(move|| {
        encode_slice(buf.as_slice());
    });
}
