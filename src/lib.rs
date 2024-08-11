#![no_std]

extern crate alloc;

mod inner_types;
pub mod iterators;
mod tests;

use alloc::{collections, vec::Vec};
use core::{fmt::Debug, ptr};
use inner_types::{StoreIndex, VecNode};
use iterators::{Iter, IterMut, IterP, VecCursor, VecCursorMut};

pub struct LinkedVec<T, I: StoreIndex + Copy = usize> {
    data: Vec<VecNode<T, I>>,
    head: Option<I>,
    tail: Option<I>,
}

impl<T, I: StoreIndex + Copy> LinkedVec<T, I> {
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            head: None,
            tail: None,
        }
    }

    /// Moves all elements from `other` to the end of the list.
    ///
    /// After this operation, `other` becomes empty.
    ///
    /// While in regular linked lists, this is *O*(1),
    /// this is *O*(n). It is provided only for API consistency.
    pub fn append(&mut self, other: &mut Self) {
        let mut third = Self::new();
        core::mem::swap(other, &mut third);
        self.extend(third)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_p(&self, index: usize) -> &T {
        &self.data[index].payload
    }

    pub fn get_p_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index].payload
    }

    /// Provides a reference to the front element, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in *O*(1) time.
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.head.map(|x| self.get_p(x.to_usize()))
    }

    /// Provides a mutable reference to the front element, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in *O*(1) time.
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.map(|x| self.get_p_mut(x.to_usize()))
    }

    /// Provides a reference to the back element, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in *O*(1) time.
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.tail.map(|x| self.get_p(x.to_usize()))
    }

    /// Provides a mutable reference to the back element, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in *O*(1) time.
    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|x| self.get_p_mut(x.to_usize()))
    }

    /// Inserts an element first in the linked list and last in the physical array.
    pub fn push_front(&mut self, value: T) {
        let inserted = self.push_p(value);

        // Insert at head = Insert before whatever is currently pointed to by head.
        self.insert_node_before(inserted, self.head)
    }

    /// Inserts an element last in the linked list and last in the physical array.
    pub fn push_back(&mut self, value: T) {
        let inserted: I = self.push_p(value);

        // Insert at tail = Insert after whatever is currently pointed to by tail.
        self.insert_node_after(inserted, self.tail)
    }

    /// Remove and return first element in the linked list, if any.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // head should be some because not is_empty
        let i = self.head.unwrap();
        Some(self.in_swap_remove(i.to_usize()))
    }

    /// Remove and return last element in the linked list, if any.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // tail should be some because not is_empty
        let i = self.tail.unwrap();
        Some(self.in_swap_remove(i.to_usize()))
    }

    /// Remove and return last element in the physical array, if any.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        };
        self.remove_node_p(self.len() - 1);
        // Safety: Already checked that data.len() is not empty
        Some(unsafe { self.data.pop().unwrap_unchecked().payload })
    }

    /// Remove and return the element pointed to by the index on the physical array.
    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len() {
            index_out_of_bounds(index, self.len())
        }
        self.in_swap_remove(index)
    }

    /// Provides a forward iterator.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T, I> {
        Iter::new(self)
    }

    /// Provides a forward iterator with mutable references.
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, I> {
        IterMut::new(self)
    }

    pub fn clear(&mut self) {
        // This doesn't clear in a particular order.
        // FIXME: Should it?
        self.data.clear();
        self.head = None;
        self.tail = None;
    }

    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq<T>,
    {
        self.iter().any(|e| e == x)
    }

    pub fn cursor_front(&self) -> VecCursor<'_, T, I> {
        VecCursor {
            index_la: 0,
            current_pa: self.head.map(|x| x.to_usize()),
            list: self,
        }
    }

    pub fn cursor_front_mut(&mut self) -> VecCursorMut<'_, T, I> {
        VecCursorMut {
            index_la: 0,
            current_pa: self.head.map(|x| x.to_usize()),
            list: self,
        }
    }

    pub fn cursor_back(&self) -> VecCursor<'_, T, I> {
        match self.tail {
            // list nonempty
            Some(tail) => VecCursor {
                index_la: self.len() - 1,
                current_pa: Some(tail.to_usize()),
                list: self,
            },

            // list empty
            None => VecCursor {
                index_la: 0,
                current_pa: None,
                list: self,
            },
        }
    }

    pub fn cursor_back_mut(&mut self) -> VecCursorMut<'_, T, I> {
        match self.tail {
            // list nonempty
            Some(tail) => VecCursorMut {
                index_la: self.len() - 1,
                current_pa: Some(tail.to_usize()),
                list: self,
            },

            // list empty
            None => VecCursorMut {
                index_la: 0,
                current_pa: None,
                list: self,
            },
        }
    }

    /// Swaps two elements in the slice.
    ///
    /// If `a` equals to `b`, it's guaranteed that elements won't change value.
    ///
    /// # Arguments
    ///
    /// * a - The index of the first element
    /// * b - The index of the second element
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    pub fn swap_p(&mut self, a: usize, b: usize) {
        let pa = ptr::addr_of_mut!(self.data[a].payload);
        let pb = ptr::addr_of_mut!(self.data[b].payload);
        // SAFETY: `pa` and `pb` have been created from safe mutable references and refer
        // to elements in the slice and therefore are guaranteed to be valid and aligned.
        // Note that accessing the elements behind `a` and `b` is checked and will
        // panic when out of bounds.
        unsafe {
            ptr::swap(pa, pb);
        }
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted.
    /// The collection may reserve more space to speculatively avoid
    /// frequent reallocations. After calling `try_reserve`, capacity will be
    /// greater than or equal to `self.len() + additional` if it returns
    /// `Ok(())`. Does nothing if capacity is already sufficient. This method
    /// preserves the contents even if an error occurs.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), collections::TryReserveError> {
        if I::MAX_USIZE.saturating_add(1) - self.len() < additional {
            // A hacky way to instantiate TryReserveErrorKind::CapacityOverflow
            self.data.try_reserve(usize::MAX)
        } else {
            self.data.try_reserve(additional)
        }
    }

    fn push_p(&mut self, value: T) -> I {
        let start_len = self.len();
        if start_len > I::MAX_USIZE {
            capacity_overflow()
        }
        self.data.push(VecNode::new(value));

        // Safety: Already checked that start_len <= MAX_USIZE
        unsafe { I::from_usize_unchecked(start_len) }
    }

    fn in_swap_remove(&mut self, index: usize) -> T {
        self.remove_node_p(index);
        let payload;
        if index != self.len() - 1 {
            payload = self.data.swap_remove(index).payload;
            self.move_node_p(index);
        } else {
            payload = self.data.remove(index).payload;
        }
        payload
    }

    /// Ensure the node in the new spots referants are pointing back.
    fn move_node_p(&mut self, index: usize) {
        let stored = Some(I::from_usize(index));
        self.set_next(self.data[index].prev, stored);
        self.set_prev(self.data[index].next, stored);
    }

    fn insert_node_before(&mut self, inserted: I, target: Option<I>) {
        let other = self.get_prev(target);
        self.pair(other, Some(inserted));
        self.pair(Some(inserted), target);
    }

    fn insert_node_after(&mut self, inserted: I, target: Option<I>) {
        let other = self.get_next(target);
        self.pair(target, Some(inserted));
        self.pair(Some(inserted), other);
    }

    fn remove_node_p(&mut self, target: usize) {
        self.pair(self.data[target].prev, self.data[target].next);
    }

    /// Gets `next` of the indexed node or `head` if `None`.
    fn get_next(&self, target: Option<I>) -> Option<I> {
        match target {
            Some(i) => self.data[i.to_usize()].next,
            None => self.head,
        }
    }

    /// Gets `prev` of the indexed node or `tail` if `None`.
    fn get_prev(&self, target: Option<I>) -> Option<I> {
        match target {
            Some(i) => self.data[i.to_usize()].prev,
            None => self.tail,
        }
    }

    /// Sets `next` of the indexed node or `head` if `None`.
    fn set_next(&mut self, target: Option<I>, value: Option<I>) {
        if let Some(i) = target {
            self.data[i.to_usize()].next = value
        } else {
            self.head = value
        }
    }

    /// Sets `prev` of the indexed node or `tail` if `None`.
    fn set_prev(&mut self, target: Option<I>, value: Option<I>) {
        if let Some(i) = target {
            self.data[i.to_usize()].prev = value
        } else {
            self.tail = value
        }
    }

    fn pair(&mut self, first: Option<I>, second: Option<I>) {
        self.set_next(first, second);
        self.set_prev(second, first);
    }
}

impl<T, I: StoreIndex> Default for LinkedVec<T, I>
where
    I: Copy + TryFrom<usize, Error: Debug> + Into<usize>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, I: StoreIndex + Copy> Clone for LinkedVec<T, I> {
    fn clone(&self) -> Self {
        let mut ret = Self::new();
        ret.clone_from(self);
        ret
    }

    fn clone_from(&mut self, source: &Self) {
        self.head = source.head;
        self.tail = source.tail;

        self.data.clear();
        self.data.extend(source.data.iter().map(|x| x.not_clone()));
    }
}

impl<T: PartialOrd, I: StoreIndex + Copy> PartialEq for LinkedVec<T, I> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: PartialOrd, I: StoreIndex + Copy> PartialOrd for LinkedVec<T, I> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Debug, I: StoreIndex + Copy> Debug for LinkedVec<T, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // FIXME: Should the format be changed?
        f.debug_map()
            .entries(IterP::new(self).map(|i| (i, self.get_p(i))))
            .finish()
    }
}

#[inline(never)]
fn index_out_of_bounds(index: impl Into<usize>, len: usize) -> ! {
    let index: usize = index.into();
    panic!("index (is {index}) should be < or <= len (is {len})");
}

#[cold]
fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}
