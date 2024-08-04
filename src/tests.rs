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

#[test]
fn len_push_pop() {
    let mut obj = LinkedVec::<isize>::new();
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