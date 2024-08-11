use alloc::borrow::ToOwned;
use core::{borrow::Borrow, fmt::Debug};
use nonmax;

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
    ($int_max1:expr, $int_max2:expr) => {{
        let m1 = $int_max1 as u128;
        let m2 = $int_max2 as u128;
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

            const MAX_USIZE: usize = min_max!(Self::MAX, usize::MAX);

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

macro_rules! storeindex_for_nonmax {
    ($prim:ty, $impor:ty) => {
        impl StoreIndex for $impor {
            type Error = nonmax::TryFromIntError;

            const MAX_USIZE: usize = min_max!(Self::MAX.get(), usize::MAX);

            fn to_usize(&self) -> usize {
                usize::try_from(self.get()).unwrap()
            }

            unsafe fn to_usize_unchecked(&self) -> usize {
                // Safety: Caller ensures self came from try_from_usize
                // or from_usize_unchecked
                unsafe { debug_unwrap!(usize::try_from(self.get())) }
            }

            fn try_from_usize(value: usize) -> Result<Self, Self::Error> {
                let intermediate = Self::try_from(value as $prim)?;
                Ok(Self::try_from(intermediate)?)
            }

            #[cfg(not(debug_assertions))]
            unsafe fn from_usize_unchecked(value: usize) -> Self {
                // Safety: Caller ensures value <= MAX_USIZE, which is
                // in the range of Self. Self's MIN is at most 0.
                unsafe { Self::new_unchecked(value as $prim) }
            }
        }
    };
}

storeindex_for_nonmax!(i8, nonmax::NonMaxI8);
storeindex_for_nonmax!(i16, nonmax::NonMaxI16);
storeindex_for_nonmax!(i32, nonmax::NonMaxI32);
storeindex_for_nonmax!(i64, nonmax::NonMaxI64);
storeindex_for_nonmax!(i128, nonmax::NonMaxI128);
storeindex_for_nonmax!(isize, nonmax::NonMaxIsize);
storeindex_for_nonmax!(u8, nonmax::NonMaxU8);
storeindex_for_nonmax!(u16, nonmax::NonMaxU16);
storeindex_for_nonmax!(u32, nonmax::NonMaxU32);
storeindex_for_nonmax!(u64, nonmax::NonMaxU64);
storeindex_for_nonmax!(u128, nonmax::NonMaxU128);
storeindex_for_nonmax!(usize, nonmax::NonMaxUsize);

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
