#![feature(test)]

extern crate test;

use std::collections::BTreeMap as Map;

use test::{black_box, Bencher};

#[bench]
fn insert(b: &mut Bencher) {
    b.iter(|| {
        let mut map = Map::new();

        let mut num = 0u64;
        for _ in 0..1_000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            map.insert(num, !num);
        }
    });
}

#[bench]
fn iter(b: &mut Bencher) {
    let mut map = Map::new();

    let mut num = 0u64;
    for _ in 0..1_000 {
        num = num.wrapping_mul(17).wrapping_add(255);
        map.insert(num, !num);
    }

    b.iter(|| {
        for x in map.iter() {
            black_box(x);
        }
    });
}

#[bench]
fn rev_iter(b: &mut Bencher) {
    let mut map = Map::new();

    let mut num = 0u64;
    for _ in 0..1_000 {
        num = num.wrapping_mul(17).wrapping_add(255);
        map.insert(num, !num);
    }

    b.iter(|| {
        for x in map.iter().rev() {
            black_box(x);
        }
    });
}

#[bench]
fn lookup(b: &mut Bencher) {
    let mut map = Map::new();

    let mut num = 0u64;
    for _ in 0..1_000 {
        num = num.wrapping_mul(17).wrapping_add(255);
        map.insert(num, !num);
    }

    b.iter(|| {
        let mut num = 0u64;

        for _ in 0..1_000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            black_box(map.get(&num));
        }
    });
}

#[bench]
fn insert_remove(b: &mut Bencher) {
    b.iter(|| {
        let mut map = Map::new();

        let mut num = 0u64;
        for _ in 0..1_000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            map.insert(num, !num);
        }

        let mut num = 0u64;
        for _ in 0..1_000 {
            num = num.wrapping_mul(17).wrapping_add(255);
            black_box(map.remove(&num).unwrap());
        }
    });
}
