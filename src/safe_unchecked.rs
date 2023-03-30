use core::slice::SliceIndex;

pub(crate) trait SafeUnchecked<T> {
    fn get_safe_unchecked<I>(&self, index: I) -> &<I as SliceIndex<[T]>>::Output
    where
        I: SliceIndex<[T]>;
}

impl<T> SafeUnchecked<T> for [T] {
    #[inline(always)]
    fn get_safe_unchecked<I>(&self, index: I) -> &<I as SliceIndex<[T]>>::Output
    where
        I: SliceIndex<[T]>,
    {
        if cfg!(debug_assertions) {
            &self[index]
        } else {
            unsafe { self.get_unchecked(index) }
        }
    }
}
