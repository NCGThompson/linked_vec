mod inner_types;
mod tests;

use inner_types::VecNode;

#[derive(Debug)]
pub struct LinkedVec<T> {
    data: Vec<VecNode<T, usize>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<T> LinkedVec<T> {
    pub const fn new() -> Self {
        LinkedVec {
            data: Vec::new(),
            head: None,
            tail: None,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts an element first in the linked list and last in the physical array.
    pub fn push_front(&mut self, value: T) {
        self.data.push(VecNode::new(value));

        // Insert at head = Insert before whatever is currently pointed to by head.
        self.insert_node_before(self.len() - 1, self.head)
    }

    /// Inserts an element last in the linked list and last in the physical array.
    pub fn push_back(&mut self, value: T) {
        self.data.push(VecNode::new(value));

        // Insert at tail = Insert after whatever is currently pointed to by tail.
        self.insert_node_after(self.len() - 1, self.tail)
    }

    /// Remove and return first element in the linked list, if any.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // head should be some because not is_empty
        let i = self.head.unwrap();
        Some(self.swap_remove(i))
    }

    /// Remove and return last element in the linked list, if any.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // tail should be some because not is_empty
        let i = self.tail.unwrap();
        Some(self.swap_remove(i))
    }

    /// Remove and return last element in the physical array, if any.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // subtraction shouldn't wrap because not is_empty
        let i = self.len() - 1;
        Some(self.swap_remove(i))
    }

    /// Remove and return the element pointed to by the index on the physical array.
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.remove_node_l(index);
        let payload;
        if index != self.len() - 1 {
            payload = self.data.swap_remove(index).payload;
            self.move_node(index);
        } else {
            payload = self.data.remove(index).payload;
        }
        payload
    }

    /// Ensure the node in the new spots referants are pointing back.
    fn move_node(&mut self, new: usize) {
        self.set_next(self.data[new].prev, Some(new));
        self.set_prev(self.data[new].next, Some(new));
    }

    fn insert_node_before(&mut self, inserted: usize, target: Option<usize>) {
        let other = self.get_prev(target);
        self.data[inserted].prev = other;
        self.data[inserted].next = target;
        self.move_node(inserted);
    }

    fn insert_node_after(&mut self, inserted: usize, target: Option<usize>) {
        let other = self.get_next(target);
        self.data[inserted].next = other;
        self.data[inserted].prev = target;
        self.move_node(inserted);
    }

    fn remove_node_l(&mut self, target: usize) {
        self.set_prev(self.data[target].next, self.data[target].prev);
        self.set_next(self.data[target].prev, self.data[target].next);
    }

    /// Gets `next` of the indexed node or `head` if `None`.
    fn get_next(&mut self, target: Option<usize>) -> Option<usize> {
        match target {
            Some(i) => self.data[i].next,
            None => self.head,
        }
    }

    /// Gets `prev` of the indexed node or `tail` if `None`.
    fn get_prev(&mut self, target: Option<usize>) -> Option<usize> {
        match target {
            Some(i) => self.data[i].prev,
            None => self.tail,
        }
    }

    /// Sets `next` of the indexed node or `head` if `None`.
    fn set_next(&mut self, target: Option<usize>, value: Option<usize>) {
        if let Some(i) = target {
            self.data[i].next = value
        } else {
            self.head = value
        }
    }

    /// Sets `prev` of the indexed node or `tail` if `None`.
    fn set_prev(&mut self, target: Option<usize>, value: Option<usize>) {
        if let Some(i) = target {
            self.data[i].prev = value
        } else {
            self.tail = value
        }
    }
}

impl<T> Default for LinkedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for LinkedVec<T> {
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

impl<A> Extend<A> for LinkedVec<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        let it = iter.into_iter();

        let l = it.size_hint().0;
        if self.data.try_reserve(l).is_err() {
            self.data.reserve_exact(l);
        }

        for v in it {
            self.push_back(v);
        }
    }
}
