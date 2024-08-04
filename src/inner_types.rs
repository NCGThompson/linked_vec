use std::borrow::Borrow;

#[derive(Debug, Default)]
pub(super) struct VecNode<T, I = usize> {
    pub payload: T,
    pub next: Option<I>,
    pub prev: Option<I>,
}

impl<T, I> VecNode<T, I> {
    pub const fn new(payload: T) -> Self {
        VecNode {
            payload,
            next: None,
            prev: None,
        }
    }
}

impl<T: ToOwned, I> ToOwned for VecNode<T, I>
where
    VecNode<T::Owned, I>: Borrow<VecNode<T, I>>,
{
    type Owned = VecNode<T::Owned, I>;
    fn to_owned(&self) -> Self::Owned {
        Self::Owned::new(self.payload.to_owned())
    }
}

impl<T: Clone, I: Clone> VecNode<T, I> {
    /// VecNode shouldn't implement Clone because
    /// fields like next, etc. are supposed to be 1-to-1
    /// with the referrent. However, we still need something to
    /// replace the clone method for when we want to clone
    /// collectios with multiple nodes (LinkedVec).
    pub fn not_clone(&self) -> Self {
        Self {
            payload: self.payload.clone(),
            next: self.next.clone(),
            prev: self.prev.clone(),
        }
    }
}
