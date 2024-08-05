use std::{borrow::Borrow, fmt::Debug};

// pub(crate) fn debug_unwrap_unchecked(res)

macro_rules! debug_unwrap {
    ($result:expr) => {
        if cfg!(debug_assertions) {
            $result.unwrap()
        } else {
            $result.unwrap_unchecked()
        }
    };
}

macro_rules! min_max {
    ($int_type1:ty, $int_type2:ty) => {{
        let m1 = <$int_type1>::MAX as u128;
        let m2 = <$int_type2>::MAX as u128;
        if m1 < m2 {
            m1 as _
        } else {
            m2 as _
        }
    }};
}

/// Can represent any usize up to a certain max value
pub trait StoreIndex: Sized {
    type Error: Debug;

    const MAX_USIZE: usize;

    /// The maximum usize the struct can consistently represent.
    /// Must always return the same value for a build.
    fn get_max() -> usize {
        Self::MAX_USIZE
    }

    /// May panic or give incorrect results only if value was not correctly
    /// instantiated with a usize in range, and was not created with try_from_usize
    fn to_usize(&self) -> usize;

    /// May lead to undefined behavior only if value was not correctly
    /// instantiated with a usize in range, and was not created with try_from_usize
    unsafe fn to_usize_unchecked(&self) -> usize {
        self.to_usize()
    }

    /// May not panic. Must succeed if value <= get_max.
    fn try_from_usize(value: usize) -> Result<Self, Self::Error>;

    /// May panic or give incorrect results only if value > get_max.
    fn from_usize(value: usize) -> Self {
        Self::try_from_usize(value).unwrap()
    }

    /// May lead to undefined behavior only if value > get_max.
    unsafe fn from_usize_unchecked(value: usize) -> Self {
        Self::from_usize(value)
    }
}

macro_rules! storeindex_for_prim {
    ($impor:ty) => {
        impl StoreIndex for $impor {
            type Error = <Self as TryFrom<usize>>::Error;

            const MAX_USIZE: usize = min_max!(Self, usize);

            fn to_usize(&self) -> usize {
                usize::try_from(*self).unwrap()
            }

            unsafe fn to_usize_unchecked(&self) -> usize {
                // Safety: Caller ensures self came from try_from_usize
                // or from_usize_unchecked
                unsafe { debug_unwrap!(usize::try_from(*self)) }
            }

            fn try_from_usize(value: usize) -> Result<Self, Self::Error> {
                Self::try_from(value)
            }

            unsafe fn from_usize_unchecked(value: usize) -> Self {
                // Safety: Caller ensures value <= MAX_USIZE, which is
                // in the range of Self. Self's MIN is at most 0.
                unsafe { debug_unwrap!(Self::try_from(value)) }
            }
        }
    };
}

storeindex_for_prim!(i8);
storeindex_for_prim!(i16);
storeindex_for_prim!(i32);
storeindex_for_prim!(i64);
storeindex_for_prim!(i128);
storeindex_for_prim!(isize);
storeindex_for_prim!(u8);
storeindex_for_prim!(u16);
storeindex_for_prim!(u32);
storeindex_for_prim!(u64);
storeindex_for_prim!(u128);
storeindex_for_prim!(usize);

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
