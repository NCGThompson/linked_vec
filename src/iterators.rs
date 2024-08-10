use crate::{inner_types::StoreIndex, LinkedVec};

#[derive(Debug)]
pub struct VecCursor<'a, T: 'a, I: Copy + StoreIndex> {
    index_la: usize,
    current_pa: Option<usize>, // Optionally replace usize with I
    list: &'a LinkedVec<T, I>,
}

impl<'a, T: 'a, I: Copy + StoreIndex> VecCursor<'a, T, I> {
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
                self.current_pa = self.list.data[current].next.map(|x| x.to_usize());
                self.index_la = self
                    .index_la
                    .checked_sub(1)
                    .unwrap_or_else(|| self.list.len());
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
        let mut next: Self = self.clone();
        next.move_prev();
        next.current()
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
        let Self { index_la, current_pa, list } = *self;

        // Create new VecCursor with individual variables.
        // `foo` is short for `foo: foo`
        Self { index_la, current_pa, list }
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
        let Self { index_la, current_pa, list } = *self;

        // Create new NonEmptyVecCursor with individual variables.
        // `foo` is short for `foo: foo`
        Self { index_la, current_pa, list }
    }
}
