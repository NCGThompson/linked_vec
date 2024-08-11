#![cfg(test)]
mod std_stolen_tests;

use alloc::borrow::ToOwned as _;
use core::mem;

use super::*;

#[test]
fn test_vecnode() {
    let mut obj = VecNode::<isize>::default();
    assert_eq!(obj.payload, 0);
    assert_eq!(obj.next, None);
    assert_eq!(obj.prev, None);

    obj.next = Some(1);
    assert_eq!(obj.not_clone().next, Some(1));
    assert_eq!(obj.to_owned().next, None);
}

fn single_len_push_pop<I: StoreIndex + Copy>() {
    let mut obj = LinkedVec::<isize, I>::new();
    assert_eq!(obj.len(), 0);
    obj.push_back(3);
    assert_eq!(obj.len(), 1);
    obj.push_back(4);
    assert_eq!(obj.len(), 2);
    obj.push_back(5);
    assert_eq!(obj.len(), 3);

    assert_eq!(obj.pop(), Some(5));
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.pop_back(), Some(4));
    assert_eq!(obj.len(), 1);
    assert_eq!(obj.pop(), Some(3));
    assert_eq!(obj.len(), 0);
    assert_eq!(obj.pop(), None);
    assert_eq!(obj.len(), 0);
}

#[test]
fn len_push_pop_subsets() {
    single_len_push_pop::<u8>();
    single_len_push_pop::<u16>();
    single_len_push_pop::<usize>();
}

#[test]
fn len_push_pop_u_superset() {
    single_len_push_pop::<u128>();
}

#[test]
fn len_push_pop_orthagonal() {
    single_len_push_pop::<i8>();
    single_len_push_pop::<i16>();
    single_len_push_pop::<isize>();
}

#[test]
fn len_push_pop_i_superset() {
    single_len_push_pop::<i128>();
}

#[test]
fn len_push_pop_nonmax() {
    single_len_push_pop::<nonmax::NonMaxU8>();
    single_len_push_pop::<nonmax::NonMaxU32>();
    single_len_push_pop::<nonmax::NonMaxU64>();
    single_len_push_pop::<nonmax::NonMaxU128>();
    single_len_push_pop::<nonmax::NonMaxUsize>();
}

#[test]
fn overflow_baseline() {
    let mut obj = LinkedVec::<i64, i8>::new();
    obj.extend(0..=127);
    assert_eq!(i8::get_max(), 127);

    let mut obj = LinkedVec::<i64, u8>::new();
    obj.extend(0..=255);
    assert_eq!(u8::get_max(), 255);

    let mut obj = LinkedVec::<i64, nonmax::NonMaxI8>::new();
    obj.extend(0..=126);
    assert_eq!(nonmax::NonMaxI8::get_max(), 126);

    let mut obj = LinkedVec::<i64, nonmax::NonMaxU8>::new();
    obj.extend(0..=254);
    assert_eq!(nonmax::NonMaxU8::get_max(), 254);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn overflow_i_a() {
    let mut obj = LinkedVec::<i64, i8>::new();
    obj.extend(0..=128);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn overflow_i_b() {
    let mut obj = LinkedVec::<i64, i8>::new();
    obj.extend(0..);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn overflow_ni_a() {
    let mut obj = LinkedVec::<i64, nonmax::NonMaxI8>::new();
    obj.extend(0..=127);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn overflow_ni_b() {
    let mut obj = LinkedVec::<i64, nonmax::NonMaxI8>::new();
    obj.extend(0..);
}

const _: () = debug_assert!(mem::size_of::<VecNode<isize, nonmax::NonMaxU32>>() == 16);
