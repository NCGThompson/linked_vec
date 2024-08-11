use alloc::vec::Vec;

pub use crate::iterators::SafeIterMut as IterMut;
use crate::{
    inner_types::{StoreIndex, VecNode},
    LinkedVec,
};

#[derive(Debug)]
pub struct VecCursor<'a, T: 'a, I: Copy + StoreIndex> {
    pub(crate) index_la: usize,
    pub(crate) current_pa: Option<usize>, // Optionally replace usize with I
    pub(crate) list: &'a LinkedVec<T, I>,
}

impl<'a, T: 'a, I: Copy + StoreIndex> VecCursor<'a, T, I> {
    /// Returns a new cursor with known index_l and index_p.
    ///
    /// index_l and index_p must both either be Some or None
    /// If they are Some, they must be corresponding index (index_l)
    /// and physical index (index_p) in list.
    #[must_use]
    pub unsafe fn new_with_index_unchecked(
        list: &'a LinkedVec<T, I>,
        index_l: Option<usize>,
        index_p: Option<usize>,
    ) -> Self {
        #[cfg(debug_assertions)]
        match (index_l, index_p) {
            (None, None) => (),
            (Some(l), Some(p)) => assert!(l < list.len() && p < list.len()),
            _ => unreachable!(),
        }

        Self {
            index_la: index_l.unwrap_or(list.len()),
            current_pa: index_p,
            list,
        }
    }

    /// Returns the cursor position within the linked list.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn index_l(&self) -> Option<usize> {
        let _ = self.current_pa?;
        Some(self.index_la)
    }

    /// Returns the cursor position within the physical array.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn index_p(&self) -> Option<usize> {
        self.current_pa
    }

    /// Returns a reference to the element that the cursor is currently
    /// pointing to.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn current(&self) -> Option<&'a T> {
        Some(self.list.get_p(self.current_pa?))
    }

    /// Returns a reference to the list that the cursor is pointing
    /// to.
    #[must_use]
    pub fn get_list(&self) -> &'a LinkedVec<T, I> {
        self.list
    }

    /// Moves the cursor to the next element of the linked list.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this will move it to
    /// the first element of the list. If it is pointing to the last
    /// element of the list, then this will move it to the "ghost" non-element.
    pub fn move_next(&mut self) {
        match self.current_pa {
            // We had no current element; the cursor was sitting at the start position
            // Next element should be the head of the list
            None => {
                self.current_pa = self.list.head.map(|x| x.to_usize());
                self.index_la = 0;
            }
            // We had a previous element, so let's go to its next
            Some(current) => {
                self.current_pa = self.list.data[current].next.map(|x| x.to_usize());
                self.index_la += 1;
            }
        }
    }

    /// Moves the cursor to the previous element of the linked list.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this will move it to
    /// the last element of the list. If it is pointing to the first
    /// element of the list, then this will move it to the "ghost" non-element.
    pub fn move_prev(&mut self) {
        match self.current_pa {
            // We had no current element; the cursor was sitting at the start position
            // Next element should be the tail of the list
            None => {
                self.current_pa = self.list.tail.map(|x| x.to_usize());
                self.index_la = self.list.len().checked_sub(1).unwrap_or(0);
            }
            // We had a previous element, so let's go to its prev
            Some(current) => {
                self.current_pa = self.list.data[current].prev.map(|x| x.to_usize());
                self.index_la = self.index_la.checked_sub(1).unwrap_or(self.list.len());
            }
        }
    }

    /// Returns a reference to the next element.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this returns
    /// the first element of the list. If it is pointing to the last
    /// element of the list then this returns `None`.
    #[must_use]
    pub fn peek_next(&self) -> Option<&'a T> {
        let mut next: Self = self.clone();
        next.move_next();
        next.current()
    }

    /// Returns a reference to the previous element.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this returns
    /// the last element of the list. If it is pointing to the first
    /// element of the list then this returns `None`.
    #[must_use]
    pub fn peek_prev(&self) -> Option<&'a T> {
        let mut prev: Self = self.clone();
        prev.move_prev();
        prev.current()
    }

    /// Equivalint to `self.list().front()`
    #[must_use]
    pub fn front(&self) -> Option<&'a T> {
        self.list.front()
    }

    /// Equivalint to `self.list().back()`
    #[must_use]
    pub fn back(&self) -> Option<&'a T> {
        self.list.back()
    }

    /// Returns a `NonEmptyVecCursor` pointing to the current element,
    /// or None if the list is empty.
    ///
    /// Changing the state of either self or the resulting cursor
    /// will not change the state of the other. If you would like
    /// to keep the state of `NonEmptyVecCursor`, then convert it back to
    /// a `VecCursor`.
    pub fn as_nonempty_cursor(&self) -> Option<NonEmptyVecCursor<'a, T, I>> {
        Some(NonEmptyVecCursor {
            index_la: self.index_la,
            current_pa: self.current_pa?,
            list: &self.list,
        })
    }
}

impl<T, I: Copy + StoreIndex> Clone for VecCursor<'_, T, I> {
    fn clone(&self) -> Self {
        // Destruct-assign self into individual variables
        // with same names as fields
        let Self {
            index_la,
            current_pa,
            list,
        } = *self;

        // Create new VecCursor with individual variables.
        // `foo` is short for `foo: foo`
        Self {
            index_la,
            current_pa,
            list,
        }
    }
}

#[derive(Debug)]
pub struct VecCursorMut<'a, T: 'a, I: Copy + StoreIndex> {
    pub(crate) index_la: usize,
    pub(crate) current_pa: Option<usize>, // Optionally replace usize with I
    pub(crate) list: &'a mut LinkedVec<T, I>,
}

impl<'a, T: 'a, I: Copy + StoreIndex> VecCursorMut<'a, T, I> {
    /// Returns a new cursor with known index_l and index_p.
    ///
    /// Usefull for upgrading from a VecCursor.
    ///
    /// index_l and index_p must both either be Some or None
    /// If they are Some, they must be corresponding index (index_l)
    /// and physical index (index_p) in list.
    #[must_use]
    pub unsafe fn new_with_index_unchecked(
        list: &'a mut LinkedVec<T, I>,
        index_l: Option<usize>,
        index_p: Option<usize>,
    ) -> Self {
        #[cfg(debug_assertions)]
        match (index_l, index_p) {
            (None, None) => (),
            (Some(l), Some(p)) => assert!(l < list.len() && p < list.len()),
            _ => unreachable!(),
        }

        Self {
            index_la: index_l.unwrap_or(list.len()),
            current_pa: index_p,
            list,
        }
    }

    /// Returns the cursor position within the linked list.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn index_l(&self) -> Option<usize> {
        let _ = self.current_pa?;
        Some(self.index_la)
    }

    /// Returns the cursor position within the physical array.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn index_p(&self) -> Option<usize> {
        self.current_pa
    }

    /// Returns a reference to the element that the cursor is currently
    /// pointing to.
    ///
    /// This returns `None` if the cursor is currently pointing to the
    /// "ghost" non-element.
    #[must_use]
    pub fn current(&mut self) -> Option<&mut T> {
        Some(self.list.get_p_mut(self.current_pa?))
    }

    /// Returns a reference to the list that the cursor is pointing
    /// to.
    #[must_use]
    pub fn get_list(&self) -> &LinkedVec<T, I> {
        self.list
    }

    /// Moves the cursor to the next element of the linked list.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this will move it to
    /// the first element of the list. If it is pointing to the last
    /// element of the list, then this will move it to the "ghost" non-element.
    pub fn move_next(&mut self) {
        match self.current_pa {
            // We had no current element; the cursor was sitting at the start position
            // Next element should be the head of the list
            None => {
                self.current_pa = self.list.head.map(|x| x.to_usize());
                self.index_la = 0;
            }
            // We had a previous element, so let's go to its next
            Some(current) => {
                self.current_pa = self.list.data[current].next.map(|x| x.to_usize());
                self.index_la += 1;
            }
        }
    }

    /// Moves the cursor to the previous element of the linked list.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this will move it to
    /// the last element of the list. If it is pointing to the first
    /// element of the list, then this will move it to the "ghost" non-element.
    pub fn move_prev(&mut self) {
        match self.current_pa {
            // We had no current element; the cursor was sitting at the start position
            // Next element should be the tail of the list
            None => {
                self.current_pa = self.list.tail.map(|x| x.to_usize());
                self.index_la = self.list.len().checked_sub(1).unwrap_or(0);
            }
            // We had a previous element, so let's go to its prev
            Some(current) => {
                self.current_pa = self.list.data[current].prev.map(|x| x.to_usize());
                self.index_la = self.index_la.checked_sub(1).unwrap_or(self.list.len());
            }
        }
    }

    /// Returns a reference to the next element.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this returns
    /// the first element of the list. If it is pointing to the last
    /// element of the list then this returns `None`.
    #[must_use]
    pub fn peek_next(&mut self) -> Option<&mut T> {
        // FIXME Maybe add a public method to not require access to list internals
        let next_p = self
            .list
            .get_next(self.current_pa.map(|x| I::from_usize(x)))?
            .to_usize();
        Some(self.list.get_p_mut(next_p))
    }

    /// Returns a reference to the previous element.
    ///
    /// If the cursor is pointing to the "ghost" non-element then this returns
    /// the last element of the list. If it is pointing to the first
    /// element of the list then this returns `None`.
    #[must_use]
    pub fn peek_prev(&mut self) -> Option<&mut T> {
        // FIXME Maybe add a public method to not require access to list internals
        let prev_p = self
            .list
            .get_prev(self.current_pa.map(|x| I::from_usize(x)))?
            .to_usize();
        Some(self.list.get_p_mut(prev_p))
    }

    /// Equivalint to `self.list().front()`
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.list.front()
    }

    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.list.front_mut()
    }

    /// Equivalint to `self.list().back()`
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.list.back()
    }

    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.list.back_mut()
    }

    #[must_use]
    pub fn as_cursor(&self) -> VecCursor<'_, T, I> {
        VecCursor {
            index_la: self.index_la,
            current_pa: self.current_pa,
            list: &self.list,
        }
    }

    /// Returns a `NonEmptyVecCursor` pointing to the current element,
    /// or None if the list is empty.
    ///
    /// Changing the state of the resulting cursor
    /// will not change the state of the mutable cursor.
    #[must_use]
    pub fn as_nonempty_cursor(&self) -> Option<NonEmptyVecCursor<'_, T, I>> {
        Some(NonEmptyVecCursor {
            index_la: self.index_la,
            current_pa: self.current_pa?,
            list: &self.list,
        })
    }
}

/// No "ghost" non-element
#[derive(Debug)]
pub struct NonEmptyVecCursor<'a, T: 'a, I: Copy + StoreIndex> {
    index_la: usize,
    current_pa: usize, // Optionally replace usize with I
    list: &'a LinkedVec<T, I>,
}

impl<'a, T: 'a, I: Copy + StoreIndex> NonEmptyVecCursor<'a, T, I> {
    /// Returns the cursor position within the linked list.
    #[must_use]
    pub fn index_l(&self) -> usize {
        self.index_la
    }
    /// Returns the cursor position within the physical array.
    #[must_use]
    pub fn index_p(&self) -> usize {
        self.current_pa
    }

    /// Returns a reference to the element that the cursor is currently
    /// pointing to.
    #[must_use]
    pub fn current(&self) -> &'a T {
        self.list.get_p(self.current_pa)
    }

    /// Moves the cursor to the next element of the linked list.
    ///
    /// If it is pointing to the last
    /// element of the list, then this will move it to the front
    /// and return false.
    pub fn move_next(&mut self) -> bool {
        match self.list.data[self.current_pa].next {
            // Next element should be the head of the list
            None => {
                self.current_pa = self.list.head.unwrap().to_usize();
                self.index_la = 0;
                false
            }
            Some(next) => {
                self.current_pa = next.to_usize();
                self.index_la += 1;
                true
            }
        }
    }

    /// Moves the cursor to the previous element of the linked list.
    ///
    /// If it is pointing to the first
    /// element of the list, then this will move it to the back
    /// and return false.
    pub fn move_prev(&mut self) -> bool {
        match self.list.data[self.current_pa].prev {
            // Next element should be the tail of the list
            None => {
                self.current_pa = self.list.tail.unwrap().to_usize();
                self.index_la = self.list.len() - 1;
                false
            }
            Some(prev) => {
                self.current_pa = prev.to_usize();
                self.index_la -= 1;
                true
            }
        }
    }

    /// Returns a `VecCursor` pointing to the current element.
    ///
    /// Changing the state of either self or the resulting cursor
    /// will not change the state of the other. If you would like
    /// to keep the state of `VecCursor`, then convert it back to
    /// a `NonEmptyVecCursor`.
    pub fn as_cursor(&self) -> VecCursor<'a, T, I> {
        VecCursor {
            index_la: self.index_la,
            current_pa: Some(self.current_pa),
            list: &self.list,
        }
    }
}

impl<T, I: Copy + StoreIndex> Clone for NonEmptyVecCursor<'_, T, I> {
    fn clone(&self) -> Self {
        // Destruct-assign self into individual variables
        // with same names as fields
        let Self {
            index_la,
            current_pa,
            list,
        } = *self;

        // Create new NonEmptyVecCursor with individual variables.
        // `foo` is short for `foo: foo`
        Self {
            index_la,
            current_pa,
            list,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T: 'a, I: Copy + StoreIndex> {
    list: &'a LinkedVec<T, I>,
    head: usize, // Could be I,
    tail: usize, // Could be I,
    len: usize,
}

impl<'a, T: 'a, I: Copy + StoreIndex> Iter<'a, T, I> {
    pub fn new(list: &'a LinkedVec<T, I>) -> Self {
        Self {
            head: list.head.map_or(0, |x| x.to_usize()),
            tail: list.tail.map_or(0, |x| x.to_usize()),
            len: list.len(),
            list,
        }
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> Iterator for Iter<'a, T, I> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_node = &self.list.data[self.head];
        self.head = last_node.next.map_or(0, |x| x.to_usize());
        Some(&last_node.payload)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> DoubleEndedIterator for Iter<'a, T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_node = &self.list.data[self.tail];
        self.tail = last_node.prev.map_or(0, |x| x.to_usize());
        Some(&last_node.payload)
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> IntoIterator for &'a LinkedVec<T, I> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, I>;

    /// Consumes the list into an iterator yielding elements by value.
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

// #[derive(Debug)]
// pub struct IterMut<'a, T: 'a, I: Copy + StoreIndex> {
//     list: &'a mut LinkedVec<T, I>,
//     head: Option<usize>, // Could be I,
//     tail: Option<usize>, // Could be I,
//     len: usize,
// }

// impl<'a, T: 'a, I: Copy + StoreIndex> Iterator for IterMut<'a, T, I> {
//     type Item = &'a mut T;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

// impl<'a, T: 'a, I: Copy + StoreIndex> DoubleEndedIterator for IterMut<'a, T, I> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

impl<'a, T: 'a, I: Copy + StoreIndex> IntoIterator for &'a mut LinkedVec<T, I> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, I>;

    // /// Consumes the list into an iterator yielding elements by value.
    // fn into_iter(self) -> Self::IntoIter {
    //     Self::IntoIter {
    //         head: self.head.map(|x| x.to_usize()),
    //         tail: self.tail.map(|x| x.to_usize()),
    //         len: self.len(),
    //         list: self, // Needs to be last
    //     }
    // }

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Exported as IterMut
#[derive(Debug)]
pub struct SafeIterMut<'a, T: 'a, I: Copy + StoreIndex> {
    ref_slice: Vec<Option<&'a mut VecNode<T, I>>>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<'a, T: 'a, I: Copy + StoreIndex> SafeIterMut<'a, T, I> {
    #[must_use]
    pub fn new(list: &'a mut LinkedVec<T, I>) -> Self {
        let len = list.len();
        let (head, tail) = match (list.head, list.tail) {
            (None, None) => (0, 0),
            (Some(h), Some(t)) => (h.to_usize(), t.to_usize()),
            _ => unreachable!(),
        };
        let ref_slice: Vec<_> = list.data.iter_mut().map(|x| Some(x)).collect();
        Self {
            ref_slice,
            head,
            tail,
            len,
        }
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> Iterator for SafeIterMut<'a, T, I> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_node = self.ref_slice[self.head].take().unwrap();
        self.head = last_node.next.map_or(0, |x| x.to_usize());
        Some(&mut last_node.payload)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> DoubleEndedIterator for SafeIterMut<'a, T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_node = self.ref_slice[self.tail].take().unwrap();
        self.tail = last_node.prev.map_or(0, |x| x.to_usize());
        Some(&mut last_node.payload)
    }
}

#[derive(Debug, Clone)]
pub struct IntoIter<T, I: Copy + StoreIndex> {
    list: LinkedVec<T, I>,
}

impl<T, I: Copy + StoreIndex> Iterator for IntoIter<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len(), Some(self.list.len()))
    }
}

impl<T, I: Copy + StoreIndex> DoubleEndedIterator for IntoIter<T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T, I: Copy + StoreIndex> IntoIterator for LinkedVec<T, I> {
    type Item = T;
    type IntoIter = IntoIter<T, I>;

    /// Consumes the list into an iterator yielding elements by value.
    fn into_iter(self) -> IntoIter<T, I> {
        IntoIter { list: self }
    }
}

impl<A, I: StoreIndex + Copy> Extend<A> for LinkedVec<A, I> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        let it = iter.into_iter();

        let l = it.size_hint().0;
        _ = self.data.try_reserve(l);

        for v in it {
            self.push_back(v);
        }
    }
}

impl<'a, A: Copy, I: StoreIndex + Copy> Extend<&'a A> for LinkedVec<A, I> {
    fn extend<T: IntoIterator<Item = &'a A>>(&mut self, iter: T) {
        let it = iter.into_iter();

        let l = it.size_hint().0;
        _ = self.data.try_reserve(l);

        for v in it {
            self.push_back(*v);
        }
    }
}

impl<A, I: StoreIndex + Copy> FromIterator<A> for LinkedVec<A, I> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IterP<'a, T: 'a, I: Copy + StoreIndex> {
    list: &'a LinkedVec<T, I>,
    head: usize, // Could be I,
    tail: usize, // Could be I,
    len: usize,
}

impl<'a, T: 'a, I: Copy + StoreIndex> IterP<'a, T, I> {
    pub fn new(list: &'a LinkedVec<T, I>) -> Self {
        Self {
            head: list.head.map_or(0, |x| x.to_usize()),
            tail: list.tail.map_or(0, |x| x.to_usize()),
            len: list.len(),
            list,
        }
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> Iterator for IterP<'a, T, I> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_index = self.head;
        self.head = self.list.data[last_index].next.map_or(0, |x| x.to_usize());
        Some(last_index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a, I: Copy + StoreIndex> DoubleEndedIterator for IterP<'a, T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }
        self.len -= 1;

        let last_index = self.tail;
        self.tail = self.list.data[last_index].prev.map_or(0, |x| x.to_usize());
        Some(last_index)
    }
}
