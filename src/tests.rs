#![cfg(test)]
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
