use crate::tests::VecNode;
use alloc::{boxed::Box, format, vec::Vec};
use rand_xoshiro::rand_core::{RngCore, SeedableRng};

use super::{LinkedVec, StoreIndex};

#[test]
fn test_basic() {
    let mut m = LinkedVec::<Box<_>>::new();
    assert_eq!(m.pop_front(), None);
    assert_eq!(m.pop_back(), None);
    assert_eq!(m.pop_front(), None);
    m.push_front(Box::new(1));
    assert_eq!(m.pop_front(), Some(Box::new(1)));
    m.push_back(Box::new(2));
    m.push_back(Box::new(3));
    assert_eq!(m.len(), 2);
    assert_eq!(m.pop_front(), Some(Box::new(2)));
    assert_eq!(m.pop_front(), Some(Box::new(3)));
    assert_eq!(m.len(), 0);
    assert_eq!(m.pop_front(), None);
    m.push_back(Box::new(1));
    m.push_back(Box::new(3));
    m.push_back(Box::new(5));
    m.push_back(Box::new(7));
    assert_eq!(m.pop_front(), Some(Box::new(1)));

    let mut n = LinkedVec::<_>::new();
    n.push_front(2);
    n.push_front(3);
    {
        assert_eq!(n.front().unwrap(), &3);
        let x = n.front_mut().unwrap();
        assert_eq!(*x, 3);
        *x = 0;
    }
    {
        assert_eq!(n.back().unwrap(), &2);
        let y = n.back_mut().unwrap();
        assert_eq!(*y, 2);
        *y = 1;
    }
    assert_eq!(n.pop_front(), Some(0));
    assert_eq!(n.pop_front(), Some(1));
}

fn generate_test() -> LinkedVec<i32> {
    list_from(&[0, 1, 2, 3, 4, 5, 6])
}

fn list_from<T: Clone, I: StoreIndex + Copy>(v: &[T]) -> LinkedVec<T, I> {
    v.iter().cloned().collect()
}

pub fn check_links<T, I: StoreIndex + Copy>(list: &LinkedVec<T, I>) {
    let mut len = 0;
    let mut last_index: Option<usize> = None;
    let mut node_index: usize;
    match list.head {
        None => {
            // tail node should also be None.
            assert!(list.tail.is_none());
            assert_eq!(0, list.len());
            return;
        }
        Some(node) => node_index = node.to_usize(),
    }

    loop {
        match (last_index, list.data[node_index].prev) {
            (None, None) => {}
            (None, _) => panic!("prev link for head"),
            (Some(p), Some(pptr)) => {
                assert_eq!(p as *const VecNode<T>, pptr.to_usize() as *const VecNode<T>);
            }
            _ => panic!("prev link is none, not good"),
        }
        match list.data[node_index].next {
            Some(next) => {
                last_index = Some(node_index);
                node_index = next.to_usize();
                len += 1;
            }
            None => {
                len += 1;
                break;
            }
        }
    }

    // verify that the tail node points to the last node.
    let tail = list.tail.expect("some tail node").to_usize();
    assert_eq!(tail, node_index);
    // check that len matches interior links.
    assert_eq!(len, list.len());
}

#[test]
fn test_append() {
    // Empty to empty
    {
        let mut m = LinkedVec::<i32>::new();
        let mut n: LinkedVec<_> = LinkedVec::new();
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 0);
        assert_eq!(n.len(), 0);
    }
    // Non-empty to empty
    {
        let mut m: LinkedVec<_> = LinkedVec::new();
        let mut n: LinkedVec<_> = LinkedVec::new();
        n.push_back(2);
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 1);
        assert_eq!(m.pop_back(), Some(2));
        assert_eq!(n.len(), 0);
        check_links(&m);
    }
    // Empty to non-empty
    {
        let mut m: LinkedVec<_> = LinkedVec::new();
        let mut n: LinkedVec<_> = LinkedVec::new();
        m.push_back(2);
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 1);
        assert_eq!(m.pop_back(), Some(2));
        check_links(&m);
    }

    // Non-empty to non-empty
    let v = Vec::from([1, 2, 3, 4, 5]);
    let u = Vec::from([9, 8, 1, 2, 3, 4, 5]);
    let mut m: LinkedVec<_> = list_from(&v);
    let mut n: LinkedVec<_> = list_from(&u);
    m.append(&mut n);
    check_links(&m);
    let mut sum = v;
    sum.extend_from_slice(&u);
    assert_eq!(sum.len(), m.len());
    for elt in sum {
        assert_eq!(m.pop_front(), Some(elt))
    }
    assert_eq!(n.len(), 0);
    // Let's make sure it's working properly, since we
    // did some direct changes to private members.
    n.push_back(3);
    assert_eq!(n.len(), 1);
    assert_eq!(n.pop_front(), Some(3));
    check_links(&n);
}

#[test]
fn test_iterator() {
    let m = generate_test();
    for (i, elt) in m.iter().enumerate() {
        assert_eq!(i as i32, *elt);
    }
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert_eq!(n.iter().next(), None);
    n.push_front(4);
    let mut it = n.iter();
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(it.next().unwrap(), &4);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(it.next(), None);
}

#[test]
fn test_iterator_clone() {
    let mut n: LinkedVec<_> = LinkedVec::new();
    n.push_back(2);
    n.push_back(3);
    n.push_back(4);
    let mut it = n.iter();
    it.next();
    let mut jt = it.clone();
    assert_eq!(it.next(), jt.next());
    assert_eq!(it.next_back(), jt.next_back());
    assert_eq!(it.next(), jt.next());
}

#[test]
fn test_iterator_double_end() {
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert_eq!(n.iter().next(), None);
    n.push_front(4);
    n.push_front(5);
    n.push_front(6);
    let mut it = n.iter();
    assert_eq!(it.size_hint(), (3, Some(3)));
    assert_eq!(it.next().unwrap(), &6);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(it.next_back().unwrap(), &4);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(it.next_back().unwrap(), &5);
    assert_eq!(it.next_back(), None);
    assert_eq!(it.next(), None);
}

#[test]
fn test_rev_iter() {
    let m = generate_test();
    for (i, elt) in m.iter().rev().enumerate() {
        assert_eq!((6 - i) as i32, *elt);
    }
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert_eq!(n.iter().rev().next(), None);
    n.push_front(4);
    let mut it = n.iter().rev();
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(it.next().unwrap(), &4);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(it.next(), None);
}

#[test]
fn test_mut_iter() {
    let mut m = generate_test();
    let mut len = m.len();
    for (i, elt) in m.iter_mut().enumerate() {
        assert_eq!(i as i32, *elt);
        len -= 1;
    }
    assert_eq!(len, 0);
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert!(n.iter_mut().next().is_none());
    n.push_front(4);
    n.push_back(5);
    let mut it = n.iter_mut();
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert!(it.next().is_some());
    assert!(it.next().is_some());
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert!(it.next().is_none());
}

#[test]
fn test_iterator_mut_double_end() {
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert!(n.iter_mut().next_back().is_none());
    n.push_front(4);
    n.push_front(5);
    n.push_front(6);
    let mut it = n.iter_mut();
    assert_eq!(it.size_hint(), (3, Some(3)));
    assert_eq!(*it.next().unwrap(), 6);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(*it.next_back().unwrap(), 4);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(*it.next_back().unwrap(), 5);
    assert!(it.next_back().is_none());
    assert!(it.next().is_none());
}

#[test]
fn test_mut_rev_iter() {
    let mut m = generate_test();
    for (i, elt) in m.iter_mut().rev().enumerate() {
        assert_eq!((6 - i) as i32, *elt);
    }
    let mut n: LinkedVec<_> = LinkedVec::new();
    assert!(n.iter_mut().rev().next().is_none());
    n.push_front(4);
    let mut it = n.iter_mut().rev();
    assert!(it.next().is_some());
    assert!(it.next().is_none());
}

#[test]
fn test_clone_from() {
    // Short cloned from long
    {
        let v = Vec::from([1, 2, 3, 4, 5]);
        let u = Vec::from([8, 7, 6, 2, 3, 4, 5]);
        let mut m: LinkedVec<_> = list_from(&v);
        let n = list_from(&u);
        m.clone_from(&n);
        check_links(&m);
        assert_eq!(m, n);
        for elt in u {
            assert_eq!(m.pop_front(), Some(elt))
        }
    }
    // Long cloned from short
    {
        let v = Vec::from([1, 2, 3, 4, 5]);
        let u = Vec::from([6, 7, 8]);
        let mut m: LinkedVec<_> = list_from(&v);
        let n = list_from(&u);
        m.clone_from(&n);
        check_links(&m);
        assert_eq!(m, n);
        for elt in u {
            assert_eq!(m.pop_front(), Some(elt))
        }
    }
    // Two equal length lists
    {
        let v = Vec::from([1, 2, 3, 4, 5]);
        let u = Vec::from([9, 8, 1, 2, 3]);
        let mut m: LinkedVec<_> = list_from(&v);
        let n = list_from(&u);
        m.clone_from(&n);
        check_links(&m);
        assert_eq!(m, n);
        for elt in u {
            assert_eq!(m.pop_front(), Some(elt))
        }
    }
}

#[test]
fn test_eq() {
    let mut n = list_from(&[]);
    let mut m: LinkedVec<_> = list_from(&[]);
    assert!(n == m);
    n.push_front(1);
    assert!(n != m);
    m.push_back(1);
    assert!(n == m);

    let n: LinkedVec<_> = list_from(&[2, 3, 4]);
    let m = list_from(&[1, 2, 3]);
    assert!(n != m);
}

#[test]
fn test_ord() {
    let n: LinkedVec<_> = list_from(&[]);
    let m = list_from(&[1, 2, 3]);
    assert!(n < m);
    assert!(m > n);
    assert!(n <= n);
    assert!(n >= n);
}

#[test]
fn test_ord_nan() {
    let nan = 0.0f64 / 0.0;
    let n: LinkedVec<_> = list_from(&[nan]);
    let m = list_from(&[nan]);
    assert!(!(n < m));
    assert!(!(n > m));
    assert!(!(n <= m));
    assert!(!(n >= m));

    let n: LinkedVec<_> = list_from(&[nan]);
    let one = list_from(&[1.0f64]);
    assert!(!(n < one));
    assert!(!(n > one));
    assert!(!(n <= one));
    assert!(!(n >= one));

    let u: LinkedVec<_> = list_from(&[1.0f64, 2.0, nan]);
    let v = list_from(&[1.0f64, 2.0, 3.0]);
    assert!(!(u < v));
    assert!(!(u > v));
    assert!(!(u <= v));
    assert!(!(u >= v));

    let s: LinkedVec<_> = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
    let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
    assert!(!(s < t));
    assert!(s > one);
    assert!(!(s <= one));
    assert!(s >= one);
}

fn fuzz_test(sz: i32, rng: &mut impl RngCore) {
    let mut m: LinkedVec<_> = LinkedVec::new();
    let mut v = Vec::from([]);
    for i in 0..sz {
        check_links(&m);
        let r: u8 = rng.next_u32() as u8;
        match r % 6 {
            0 => {
                m.pop_back();
                v.pop();
            }
            1 => {
                if !v.is_empty() {
                    m.pop_front();
                    v.remove(0);
                }
            }
            2 | 4 => {
                m.push_front(-i);
                v.insert(0, -i);
            }
            3 | 5 | _ => {
                m.push_back(i);
                v.push(i);
            }
        }
    }

    check_links(&m);

    let mut i = 0;
    for (a, &b) in m.into_iter().zip(&v) {
        i += 1;
        assert_eq!(a, b);
    }
    assert_eq!(i, v.len());
}

#[test]
fn test_fuzz() {
    let mut rng = rand_xoshiro::Xoroshiro128StarStar::seed_from_u64(127);
    for _ in 0..25 {
        fuzz_test(3, &mut rng);
        fuzz_test(16, &mut rng);
        #[cfg(not(miri))] // Miri is too slow
        fuzz_test(189, &mut rng);
    }
}

#[test]
fn test_show() {
    let mut list: LinkedVec<_> = (0..10).collect();
    list.pop_front();
    list.push_front(0);
    assert_eq!(
        format!("{list:?}"),
        "{9: 0, 1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 6, 7: 7, 8: 8, 0: 9}"
    );

    let list: LinkedVec<_> = ["just", "one", "test", "more"].into_iter().collect();
    assert_eq!(
        format!("{list:?}"),
        "{0: \"just\", 1: \"one\", 2: \"test\", 3: \"more\"}"
    );
}

// #[test]
// fn extract_if_test() {
//     let mut m: LinkedVec<u32> = LinkedVec::new();
//     m.extend(&[1, 2, 3, 4, 5, 6]);
//     let deleted = m.extract_if(|v| *v < 4).collect::<Vec<_>>();

//     check_links(&m);

//     assert_eq!(deleted, &[1, 2, 3]);
//     assert_eq!(m.into_iter().collect::<Vec<_>>(), &[4, 5, 6]);
// }

// #[test]
// fn drain_to_empty_test() {
//     let mut m: LinkedVec<u32> = LinkedVec::new();
//     m.extend(&[1, 2, 3, 4, 5, 6]);
//     let deleted = m.extract_if(|_| true).collect::<Vec<_>>();

//     check_links(&m);

//     assert_eq!(deleted, &[1, 2, 3, 4, 5, 6]);
//     assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
// }

#[test]
fn test_cursor_move_peek() {
    let mut m: LinkedVec<u32> = LinkedVec::new();
    m.extend(&[1, 2, 3, 4, 5, 6]);
    let mut cursor = m.cursor_front();
    assert_eq!(cursor.current(), Some(&1));
    assert_eq!(cursor.peek_next(), Some(&2));
    assert_eq!(cursor.peek_prev(), None);
    assert_eq!(cursor.index_l(), Some(0));
    cursor.move_prev();
    assert_eq!(cursor.current(), None);
    assert_eq!(cursor.peek_next(), Some(&1));
    assert_eq!(cursor.peek_prev(), Some(&6));
    assert_eq!(cursor.index_l(), None);
    cursor.move_next();
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&2));
    assert_eq!(cursor.peek_next(), Some(&3));
    assert_eq!(cursor.peek_prev(), Some(&1));
    assert_eq!(cursor.index_l(), Some(1));

    let mut cursor = m.cursor_back();
    assert_eq!(cursor.current(), Some(&6));
    assert_eq!(cursor.peek_next(), None);
    assert_eq!(cursor.peek_prev(), Some(&5));
    assert_eq!(cursor.index_l(), Some(5));
    cursor.move_next();
    assert_eq!(cursor.current(), None);
    assert_eq!(cursor.peek_next(), Some(&1));
    assert_eq!(cursor.peek_prev(), Some(&6));
    assert_eq!(cursor.index_l(), None);
    cursor.move_prev();
    cursor.move_prev();
    assert_eq!(cursor.current(), Some(&5));
    assert_eq!(cursor.peek_next(), Some(&6));
    assert_eq!(cursor.peek_prev(), Some(&4));
    assert_eq!(cursor.index_l(), Some(4));

    let mut m: LinkedVec<u32> = LinkedVec::new();
    m.extend(&[1, 2, 3, 4, 5, 6]);
    let mut cursor = m.cursor_front_mut();
    assert_eq!(cursor.current(), Some(&mut 1));
    assert_eq!(cursor.peek_next(), Some(&mut 2));
    assert_eq!(cursor.peek_prev(), None);
    assert_eq!(cursor.index_l(), Some(0));
    cursor.move_prev();
    assert_eq!(cursor.current(), None);
    assert_eq!(cursor.peek_next(), Some(&mut 1));
    assert_eq!(cursor.peek_prev(), Some(&mut 6));
    assert_eq!(cursor.index_l(), None);
    cursor.move_next();
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&mut 2));
    assert_eq!(cursor.peek_next(), Some(&mut 3));
    assert_eq!(cursor.peek_prev(), Some(&mut 1));
    assert_eq!(cursor.index_l(), Some(1));
    let mut cursor2 = cursor.as_cursor();
    assert_eq!(cursor2.current(), Some(&2));
    assert_eq!(cursor2.index_l(), Some(1));
    cursor2.move_next();
    assert_eq!(cursor2.current(), Some(&3));
    assert_eq!(cursor2.index_l(), Some(2));
    assert_eq!(cursor.current(), Some(&mut 2));
    assert_eq!(cursor.index_l(), Some(1));

    let mut m: LinkedVec<u32> = LinkedVec::new();
    m.extend(&[1, 2, 3, 4, 5, 6]);
    let mut cursor = m.cursor_back_mut();
    assert_eq!(cursor.current(), Some(&mut 6));
    assert_eq!(cursor.peek_next(), None);
    assert_eq!(cursor.peek_prev(), Some(&mut 5));
    assert_eq!(cursor.index_l(), Some(5));
    cursor.move_next();
    assert_eq!(cursor.current(), None);
    assert_eq!(cursor.peek_next(), Some(&mut 1));
    assert_eq!(cursor.peek_prev(), Some(&mut 6));
    assert_eq!(cursor.index_l(), None);
    cursor.move_prev();
    cursor.move_prev();
    assert_eq!(cursor.current(), Some(&mut 5));
    assert_eq!(cursor.peek_next(), Some(&mut 6));
    assert_eq!(cursor.peek_prev(), Some(&mut 4));
    assert_eq!(cursor.index_l(), Some(4));
    let mut cursor2 = cursor.as_cursor();
    assert_eq!(cursor2.current(), Some(&5));
    assert_eq!(cursor2.index_l(), Some(4));
    cursor2.move_prev();
    assert_eq!(cursor2.current(), Some(&4));
    assert_eq!(cursor2.index_l(), Some(3));
    assert_eq!(cursor.current(), Some(&mut 5));
    assert_eq!(cursor.index_l(), Some(4));
}

// #[test]
// fn test_cursor_mut_insert() {
//     let mut m: LinkedVec<u32> = LinkedVec::new();
//     m.extend(&[1, 2, 3, 4, 5, 6]);
//     let mut cursor = m.cursor_front_mut();
//     cursor.insert_before(7);
//     cursor.insert_after(8);
//     check_links(&m);
//     assert_eq!(
//         m.iter().cloned().collect::<Vec<_>>(),
//         &[7, 1, 8, 2, 3, 4, 5, 6]
//     );
//     let mut cursor = m.cursor_front_mut();
//     cursor.move_prev();
//     cursor.insert_before(9);
//     cursor.insert_after(10);
//     check_links(&m);
//     assert_eq!(
//         m.iter().cloned().collect::<Vec<_>>(),
//         &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]
//     );
//     let mut cursor = m.cursor_front_mut();
//     cursor.move_prev();
//     assert_eq!(cursor.remove_current(), None);
//     cursor.move_next();
//     cursor.move_next();
//     assert_eq!(cursor.remove_current(), Some(7));
//     cursor.move_prev();
//     cursor.move_prev();
//     cursor.move_prev();
//     assert_eq!(cursor.remove_current(), Some(9));
//     cursor.move_next();
//     assert_eq!(cursor.remove_current(), Some(10));
//     check_links(&m);
//     assert_eq!(
//         m.iter().cloned().collect::<Vec<_>>(),
//         &[1, 8, 2, 3, 4, 5, 6]
//     );
//     let mut cursor = m.cursor_front_mut();
//     let mut p: LinkedVec<u32> = LinkedVec::new();
//     p.extend(&[100, 101, 102, 103]);
//     let mut q: LinkedVec<u32> = LinkedVec::new();
//     q.extend(&[200, 201, 202, 203]);
//     cursor.splice_after(p);
//     cursor.splice_before(q);
//     check_links(&m);
//     assert_eq!(
//         m.iter().cloned().collect::<Vec<_>>(),
//         &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
//     );
//     let mut cursor = m.cursor_front_mut();
//     cursor.move_prev();
//     let tmp = cursor.split_before();
//     assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
//     m = tmp;
//     let mut cursor = m.cursor_front_mut();
//     cursor.move_next();
//     cursor.move_next();
//     cursor.move_next();
//     cursor.move_next();
//     cursor.move_next();
//     cursor.move_next();
//     let tmp = cursor.split_after();
//     assert_eq!(
//         tmp.into_iter().collect::<Vec<_>>(),
//         &[102, 103, 8, 2, 3, 4, 5, 6]
//     );
//     check_links(&m);
//     assert_eq!(
//         m.iter().cloned().collect::<Vec<_>>(),
//         &[200, 201, 202, 203, 1, 100, 101]
//     );
// }

// #[test]
// fn test_cursor_push_front_back() {
//     let mut ll: LinkedVec<u32> = LinkedVec::new();
//     ll.extend(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
//     let mut c = ll.cursor_front_mut();
//     assert_eq!(c.current(), Some(&mut 1));
//     assert_eq!(c.index(), Some(0));
//     c.push_front(0);
//     assert_eq!(c.current(), Some(&mut 1));
//     assert_eq!(c.peek_prev(), Some(&mut 0));
//     assert_eq!(c.index(), Some(1));
//     c.push_back(11);
//     drop(c);
//     let p = ll.cursor_back().front().unwrap();
//     assert_eq!(p, &0);
//     assert_eq!(ll, (0..12).collect());
//     check_links(&ll);
// }

// #[test]
// fn test_cursor_pop_front_back() {
//     let mut ll: LinkedVec<u32> = LinkedVec::new();
//     ll.extend(&[1, 2, 3, 4, 5, 6]);
//     let mut c = ll.cursor_back_mut();
//     assert_eq!(c.pop_front(), Some(1));
//     c.move_prev();
//     c.move_prev();
//     c.move_prev();
//     assert_eq!(c.pop_back(), Some(6));
//     let c = c.as_cursor();
//     assert_eq!(c.front(), Some(&2));
//     assert_eq!(c.back(), Some(&5));
//     assert_eq!(c.index(), Some(1));
//     drop(c);
//     assert_eq!(ll, (2..6).collect());
//     check_links(&ll);
//     let mut c = ll.cursor_back_mut();
//     assert_eq!(c.current(), Some(&mut 5));
//     assert_eq!(c.index, 3);
//     assert_eq!(c.pop_back(), Some(5));
//     assert_eq!(c.current(), None);
//     assert_eq!(c.index, 3);
//     assert_eq!(c.pop_back(), Some(4));
//     assert_eq!(c.current(), None);
//     assert_eq!(c.index, 2);
// }

#[test]
fn test_extend_ref() {
    let mut a: LinkedVec<_> = LinkedVec::new();
    a.push_back(1);

    a.extend(&[2, 3, 4]);

    assert_eq!(a.len(), 4);
    assert_eq!(a, list_from(&[1, 2, 3, 4]));

    let mut b: LinkedVec<_> = LinkedVec::new();
    b.push_back(5);
    b.push_back(6);
    a.extend(&b);

    assert_eq!(a.len(), 6);
    assert_eq!(a, list_from(&[1, 2, 3, 4, 5, 6]));
}

#[test]
fn test_extend() {
    let mut a: LinkedVec<_> = LinkedVec::new();
    a.push_back(1);
    a.extend(Vec::from([2, 3, 4])); // uses iterator

    assert_eq!(a.len(), 4);
    assert!(a.iter().eq(&[1, 2, 3, 4]));

    let b: LinkedVec<_> = [5, 6, 7].into_iter().collect();
    a.extend(b); // specializes to `append`

    assert_eq!(a.len(), 7);
    assert!(a.iter().eq(&[1, 2, 3, 4, 5, 6, 7]));
}

#[test]
fn test_contains() {
    let mut l: LinkedVec<_> = LinkedVec::new();
    l.extend(&[2, 3, 4]);

    assert!(l.contains(&3));
    assert!(!l.contains(&1));

    l.clear();

    assert!(!l.contains(&3));
}

// #[test]
// fn extract_if_empty() {
//     let mut list: LinkedVec<i32> = LinkedVec::new();

//     {
//         let mut iter = list.extract_if(|_| true);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//     }

//     assert_eq!(list.len(), 0);
//     assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([]));
// }

// #[test]
// fn extract_if_zst() {
//     let mut list: LinkedVec<_> = [(), (), (), (), ()].into_iter().collect();
//     let initial_len = list.len();
//     let mut count = 0;

//     {
//         let mut iter = list.extract_if(|_| true);
//         assert_eq!(iter.size_hint(), (0, Some(initial_len)));
//         while let Some(_) = iter.next() {
//             count += 1;
//             assert_eq!(iter.size_hint(), (0, Some(initial_len - count)));
//         }
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//     }

//     assert_eq!(count, initial_len);
//     assert_eq!(list.len(), 0);
//     assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([]));
// }

// #[test]
// fn extract_if_false() {
//     let mut list: LinkedVec<_> = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].into_iter().collect();

//     let initial_len = list.len();
//     let mut count = 0;

//     {
//         let mut iter = list.extract_if(|_| false);
//         assert_eq!(iter.size_hint(), (0, Some(initial_len)));
//         for _ in iter.by_ref() {
//             count += 1;
//         }
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//     }

//     assert_eq!(count, 0);
//     assert_eq!(list.len(), initial_len);
//     assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
// }

// #[test]
// fn extract_if_true() {
//     let mut list: LinkedVec<_> = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].into_iter().collect();

//     let initial_len = list.len();
//     let mut count = 0;

//     {
//         let mut iter = list.extract_if(|_| true);
//         assert_eq!(iter.size_hint(), (0, Some(initial_len)));
//         while let Some(_) = iter.next() {
//             count += 1;
//             assert_eq!(iter.size_hint(), (0, Some(initial_len - count)));
//         }
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//     }

//     assert_eq!(count, initial_len);
//     assert_eq!(list.len(), 0);
//     assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([]));
// }

// #[test]
// fn extract_if_complex() {
//     {
//         //                [+xxx++++++xxxxx++++x+x++]
//         let mut list = [
//             1, 2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36, 37,
//             39,
//         ]
//         .into_iter()
//         .collect::<LinkedVec<_>>();

//         let removed = list.extract_if(|x| *x % 2 == 0).collect::<Vec<_>>();
//         assert_eq!(removed.len(), 10);
//         assert_eq!(removed, Vec::from([2, 4, 6, 18, 20, 22, 24, 26, 34, 36]));

//         assert_eq!(list.len(), 14);
//         assert_eq!(
//             list.into_iter().collect::<Vec<_>>(),
//             Vec::from([1, 7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35, 37, 39])
//         );
//     }

//     {
//         // [xxx++++++xxxxx++++x+x++]
//         let mut list =
//             [2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36, 37, 39]
//                 .into_iter()
//                 .collect::<LinkedVec<_>>();

//         let removed = list.extract_if(|x| *x % 2 == 0).collect::<Vec<_>>();
//         assert_eq!(removed.len(), 10);
//         assert_eq!(removed, Vec::from([2, 4, 6, 18, 20, 22, 24, 26, 34, 36]));

//         assert_eq!(list.len(), 13);
//         assert_eq!(
//             list.into_iter().collect::<Vec<_>>(),
//             Vec::from([7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35, 37, 39])
//         );
//     }

//     {
//         // [xxx++++++xxxxx++++x+x]
//         let mut list =
//             [2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36]
//                 .into_iter()
//                 .collect::<LinkedVec<_>>();

//         let removed = list.extract_if(|x| *x % 2 == 0).collect::<Vec<_>>();
//         assert_eq!(removed.len(), 10);
//         assert_eq!(removed, Vec::from([2, 4, 6, 18, 20, 22, 24, 26, 34, 36]));

//         assert_eq!(list.len(), 11);
//         assert_eq!(
//             list.into_iter().collect::<Vec<_>>(),
//             Vec::from([7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35])
//         );
//     }

//     {
//         // [xxxxxxxxxx+++++++++++]
//         let mut list = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19]
//             .into_iter()
//             .collect::<LinkedVec<_>>();

//         let removed = list.extract_if(|x| *x % 2 == 0).collect::<Vec<_>>();
//         assert_eq!(removed.len(), 10);
//         assert_eq!(removed, Vec::from([2, 4, 6, 8, 10, 12, 14, 16, 18, 20]));

//         assert_eq!(list.len(), 10);
//         assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([1, 3, 5, 7, 9, 11, 13, 15, 17, 19]));
//     }

//     {
//         // [+++++++++++xxxxxxxxxx]
//         let mut list = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
//             .into_iter()
//             .collect::<LinkedVec<_>>();

//         let removed = list.extract_if(|x| *x % 2 == 0).collect::<Vec<_>>();
//         assert_eq!(removed.len(), 10);
//         assert_eq!(removed, Vec::from([2, 4, 6, 8, 10, 12, 14, 16, 18, 20]));

//         assert_eq!(list.len(), 10);
//         assert_eq!(list.into_iter().collect::<Vec<_>>(), Vec::from([1, 3, 5, 7, 9, 11, 13, 15, 17, 19]));
//     }
// }

#[test]
fn test_drop() {
    static mut DROPS: i32 = 0;
    struct Elem;
    impl Drop for Elem {
        fn drop(&mut self) {
            unsafe {
                DROPS += 1;
            }
        }
    }

    let mut ring: LinkedVec<_> = LinkedVec::new();
    ring.push_back(Elem);
    ring.push_front(Elem);
    ring.push_back(Elem);
    ring.push_front(Elem);
    drop(ring);

    assert_eq!(unsafe { DROPS }, 4);
}

#[test]
fn test_drop_with_pop() {
    static mut DROPS: i32 = 0;
    struct Elem;
    impl Drop for Elem {
        fn drop(&mut self) {
            unsafe {
                DROPS += 1;
            }
        }
    }

    let mut ring: LinkedVec<_> = LinkedVec::new();
    ring.push_back(Elem);
    ring.push_front(Elem);
    ring.push_back(Elem);
    ring.push_front(Elem);

    drop(ring.pop_back());
    drop(ring.pop_front());
    assert_eq!(unsafe { DROPS }, 2);

    drop(ring);
    assert_eq!(unsafe { DROPS }, 4);
}

#[test]
fn test_drop_clear() {
    static mut DROPS: i32 = 0;
    struct Elem;
    impl Drop for Elem {
        fn drop(&mut self) {
            unsafe {
                DROPS += 1;
            }
        }
    }

    let mut ring: LinkedVec<_> = LinkedVec::new();
    ring.push_back(Elem);
    ring.push_front(Elem);
    ring.push_back(Elem);
    ring.push_front(Elem);
    ring.clear();
    assert_eq!(unsafe { DROPS }, 4);

    drop(ring);
    assert_eq!(unsafe { DROPS }, 4);
}
