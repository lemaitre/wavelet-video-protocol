#![allow(clippy::missing_safety_doc)]

use std::{hash::Hash, iter::FusedIterator, marker::PhantomData, num::NonZero, ptr::NonNull};

pub const STEP_1: NonZero<isize> = unsafe { NonZero::new_unchecked(1) };

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Never {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owned<T: ?Sized>(PhantomData<Box<T>>, Never);

pub unsafe trait FromPtr {
    type Pointee;
    type State: Clone + Default + std::fmt::Debug;
    const NESTING: usize = 0;
    type Borrow<'a>: FromPtr<Pointee = Self::Pointee, State = Self::State> + 'a
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self;
    fn state_total_size(state: &Self::State) -> usize {
        _ = state;
        1
    }
}

unsafe impl<T> FromPtr for NonNull<T> {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = NonNull<T>
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        pointer
    }
}
unsafe impl<T> FromPtr for NonNull<[T]> {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = NonNull<[T]>
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        NonNull::slice_from_raw_parts(pointer, state)
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T> FromPtr for *const T {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = *const T
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        pointer.as_ptr()
    }
}
unsafe impl<T> FromPtr for *const [T] {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = *const [T]
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        std::ptr::slice_from_raw_parts(pointer.as_ptr(), state)
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T> FromPtr for *mut T {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = *const T
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        pointer.as_ptr()
    }
}
unsafe impl<T> FromPtr for *mut [T] {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = *const [T]
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        std::ptr::slice_from_raw_parts_mut(pointer.as_ptr(), state)
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T> FromPtr for &T {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        unsafe { pointer.as_ref() }
    }
}
unsafe impl<T> FromPtr for &[T] {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = &'a [T]
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        unsafe { std::slice::from_raw_parts(pointer.as_ptr(), state) }
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T> FromPtr for &mut T {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn from_ptr(mut pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        unsafe { pointer.as_mut() }
    }
}
unsafe impl<T> FromPtr for &mut [T] {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = &'a [T]
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        unsafe { std::slice::from_raw_parts_mut(pointer.as_ptr(), state) }
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T> FromPtr for Owned<T> {
    type Pointee = T;
    type State = ();
    type Borrow<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn from_ptr(_pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        panic!("Cannot move from a view where the underlying type is Owned")
    }
}
unsafe impl<T> FromPtr for Owned<[T]> {
    type Pointee = T;
    type State = usize;
    type Borrow<'a>
        = &'a [T]
    where
        Self: 'a;

    unsafe fn from_ptr(_pointer: NonNull<Self::Pointee>, _state: Self::State) -> Self {
        panic!("Cannot move from a view where the underlying type is Owned")
    }
    fn state_total_size(state: &Self::State) -> usize {
        *state
    }
}

unsafe impl<T: FromPtr> FromPtr for Strided<T> {
    type Pointee = T::Pointee;
    type State = StridedState<T::State>;
    const NESTING: usize = 1 + T::NESTING;
    type Borrow<'a>
        = Strided<T::Borrow<'a>>
    where
        Self: 'a;

    unsafe fn from_ptr(pointer: NonNull<Self::Pointee>, state: Self::State) -> Self {
        unsafe { Self::from_raw_parts(pointer, state) }
    }

    fn state_total_size(state: &Self::State) -> usize {
        let StridedState {
            len,
            stride: _,
            inner,
        } = state;
        *len * <T as FromPtr>::state_total_size(inner)
    }
}

pub unsafe trait FromPtrMut: FromPtr {
    type BorrowMut<'a>: FromPtrMut<Pointee = Self::Pointee, State = Self::State> + 'a
    where
        Self: 'a;
}

unsafe impl<T> FromPtrMut for NonNull<T>
where
    T: ?Sized,
    Self: FromPtr,
{
    type BorrowMut<'a>
        = Self
    where
        Self: 'a;
}

unsafe impl<T> FromPtrMut for *mut T
where
    T: ?Sized,
    Self: FromPtr,
{
    type BorrowMut<'a>
        = *mut T
    where
        Self: 'a;
}

unsafe impl<T> FromPtrMut for &mut T {
    type BorrowMut<'a>
        = &'a mut T
    where
        Self: 'a;
}
unsafe impl<T> FromPtrMut for &mut [T] {
    type BorrowMut<'a>
        = &'a mut [T]
    where
        Self: 'a;
}

unsafe impl<T> FromPtrMut for Owned<T> {
    type BorrowMut<'a>
        = &'a mut T
    where
        Self: 'a;
}
unsafe impl<T> FromPtrMut for Owned<[T]> {
    type BorrowMut<'a>
        = &'a mut [T]
    where
        Self: 'a;
}

unsafe impl<T: FromPtrMut> FromPtrMut for Strided<T> {
    type BorrowMut<'a>
        = Strided<T::BorrowMut<'a>>
    where
        Self: 'a;
}

pub unsafe trait RefLike<'a>: FromPtr + 'a {
    type AsPtr: PtrLike<AsRef<'a> = Self::Borrow<'a>>
        + FromPtr<Pointee = Self::Pointee, State = Self::State>;
}
pub unsafe trait MutLike<'a>: RefLike<'a> + FromPtrMut
where
    Self: RefLike<'a, AsPtr = <Self as MutLike<'a>>::AsPtr>,
{
    type AsPtr: PtrLike<AsMut<'a> = Self::BorrowMut<'a>>
        + FromPtr<Pointee = Self::Pointee, State = Self::State>;
}

pub unsafe trait PtrLike: FromPtr {
    type AsRef<'a>: RefLike<'a> + FromPtr<Pointee = Self::Pointee, State = Self::State>
    where
        Self: 'a;
    type AsMut<'a>: MutLike<'a> + FromPtr<Pointee = Self::Pointee, State = Self::State>
    where
        Self: 'a;
}

unsafe impl<'a, T> RefLike<'a> for &'a T {
    type AsPtr = NonNull<T>;
}
unsafe impl<'a, T> RefLike<'a> for &'a [T] {
    type AsPtr = NonNull<[T]>;
}
unsafe impl<'a, T: RefLike<'a> + FromPtr> RefLike<'a> for Strided<T> {
    type AsPtr = Strided<T::AsPtr>;
}

unsafe impl<'a, T> MutLike<'a> for &'a mut T {
    type AsPtr = NonNull<T>;
}
unsafe impl<'a, T> MutLike<'a> for &'a mut [T] {
    type AsPtr = NonNull<[T]>;
}
unsafe impl<'a, T: MutLike<'a> + FromPtr> MutLike<'a> for Strided<T> {
    type AsPtr = Strided<<T as MutLike<'a>>::AsPtr>;
}
unsafe impl<'a, T> RefLike<'a> for &'a mut T {
    type AsPtr = NonNull<T>;
}
unsafe impl<'a, T> RefLike<'a> for &'a mut [T] {
    type AsPtr = NonNull<[T]>;
}

unsafe impl<T> PtrLike for *const T {
    type AsRef<'a>
        = &'a T
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut T
    where
        Self: 'a;
}
unsafe impl<T> PtrLike for *const [T] {
    type AsRef<'a>
        = &'a [T]
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut [T]
    where
        Self: 'a;
}
unsafe impl<T> PtrLike for *mut T {
    type AsRef<'a>
        = &'a T
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut T
    where
        Self: 'a;
}
unsafe impl<T> PtrLike for *mut [T] {
    type AsRef<'a>
        = &'a [T]
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut [T]
    where
        Self: 'a;
}
unsafe impl<T> PtrLike for NonNull<T> {
    type AsRef<'a>
        = &'a T
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut T
    where
        Self: 'a;
}
unsafe impl<T> PtrLike for NonNull<[T]> {
    type AsRef<'a>
        = &'a [T]
    where
        Self: 'a;

    type AsMut<'a>
        = &'a mut [T]
    where
        Self: 'a;
}
unsafe impl<T: PtrLike> PtrLike for Strided<T> {
    type AsRef<'a>
        = Strided<T::AsRef<'a>>
    where
        Self: 'a;

    type AsMut<'a>
        = Strided<T::AsMut<'a>>
    where
        Self: 'a;
}

#[allow(clippy::len_without_is_empty)]
pub unsafe trait SliceLike {
    type GetItem: FromPtr;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee>;
    fn len(&self) -> usize;
    fn stride(&self) -> isize {
        std::mem::size_of::<<Self::GetItem as FromPtr>::Pointee>() as isize
    }
}

unsafe impl<T> SliceLike for NonNull<[T]> {
    type GetItem = NonNull<T>;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_ptr().cast()) }
    }

    fn len(&self) -> usize {
        (*self).len()
    }
}
unsafe impl<T, const N: usize> SliceLike for NonNull<[T; N]> {
    type GetItem = NonNull<T>;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_ptr().cast()) }
    }

    fn len(&self) -> usize {
        N
    }
}
unsafe impl<T> SliceLike for *const [T] {
    type GetItem = *const T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        NonNull::new(self.cast_mut().cast()).expect("Should not be null")
    }

    fn len(&self) -> usize {
        (*self).len()
    }
}
unsafe impl<T, const N: usize> SliceLike for *const [T; N] {
    type GetItem = *const T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        NonNull::new(self.cast_mut().cast()).expect("Should not be null")
    }

    fn len(&self) -> usize {
        N
    }
}
unsafe impl<T> SliceLike for *mut [T] {
    type GetItem = *mut T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        NonNull::new(self.cast()).expect("Should not be null")
    }

    fn len(&self) -> usize {
        (*self).len()
    }
}
unsafe impl<T, const N: usize> SliceLike for *mut [T; N] {
    type GetItem = *mut T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        NonNull::new(self.cast()).expect("Should not be null")
    }

    fn len(&self) -> usize {
        N
    }
}
unsafe impl<'a, T> SliceLike for &'a [T] {
    type GetItem = &'a T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_ptr().cast_mut()) }
    }

    fn len(&self) -> usize {
        (*self).len()
    }
}
unsafe impl<'a, T, const N: usize> SliceLike for &'a [T; N] {
    type GetItem = &'a T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_ptr().cast_mut()) }
    }

    fn len(&self) -> usize {
        N
    }
}
unsafe impl<'a, T> SliceLike for &'a mut [T] {
    type GetItem = &'a mut T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_mut_ptr()) }
    }

    fn len(&self) -> usize {
        self.iter().len()
    }
}
unsafe impl<'a, T, const N: usize> SliceLike for &'a mut [T; N] {
    type GetItem = &'a mut T;

    fn as_non_null_ptr(&mut self) -> NonNull<<Self::GetItem as FromPtr>::Pointee> {
        unsafe { NonNull::new_unchecked(self.as_mut_ptr()) }
    }

    fn len(&self) -> usize {
        N
    }
}

pub unsafe trait GridLike: SliceLike {
    type CellItem: FromPtr;
}

unsafe impl<T, const N: usize> GridLike for NonNull<[[T; N]]> {
    type CellItem = NonNull<T>;
}
unsafe impl<T, const N: usize, const M: usize> GridLike for NonNull<[[T; N]; M]> {
    type CellItem = NonNull<T>;
}
unsafe impl<T, const N: usize> GridLike for *mut [[T; N]] {
    type CellItem = *mut T;
}
unsafe impl<T, const N: usize, const M: usize> GridLike for *mut [[T; N]; M] {
    type CellItem = *mut T;
}
unsafe impl<T, const N: usize> GridLike for *const [[T; N]] {
    type CellItem = *const T;
}
unsafe impl<T, const N: usize, const M: usize> GridLike for *const [[T; N]; M] {
    type CellItem = *const T;
}
unsafe impl<'a, T, const N: usize> GridLike for &'a [[T; N]] {
    type CellItem = &'a T;
}
unsafe impl<'a, T, const N: usize, const M: usize> GridLike for &'a [[T; N]; M] {
    type CellItem = &'a T;
}
unsafe impl<'a, T, const N: usize> GridLike for &'a mut [[T; N]] {
    type CellItem = &'a mut T;
}
unsafe impl<'a, T, const N: usize, const M: usize> GridLike for &'a mut [[T; N]; M] {
    type CellItem = &'a mut T;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StridedState<T> {
    pub len: usize,
    pub stride: isize,
    pub inner: T,
}

fn transpose_state(
    len_a: &mut usize,
    stride_a: &mut isize,
    len_b: &mut usize,
    stride_b: &mut isize,
) {
    std::mem::swap(len_a, len_b);

    if *len_a == 0 || *len_b == 0 {
        *stride_a = 0;
        *stride_b = 0;
    } else {
        std::mem::swap(stride_a, stride_b);
    }
}

impl<T> StridedState<StridedState<T>> {
    pub fn transpose01(&mut self) {
        transpose_state(
            &mut self.len,
            &mut self.stride,
            &mut self.inner.len,
            &mut self.inner.stride,
        );
    }
}

impl<T> StridedState<StridedState<StridedState<T>>> {
    pub fn transpose02(&mut self) {
        transpose_state(
            &mut self.len,
            &mut self.stride,
            &mut self.inner.inner.len,
            &mut self.inner.inner.stride,
        );
    }
    pub fn transpose12(&mut self) {
        self.inner.transpose01();
    }
}
impl<T> StridedState<StridedState<StridedState<StridedState<T>>>> {
    pub fn transpose03(&mut self) {
        transpose_state(
            &mut self.len,
            &mut self.stride,
            &mut self.inner.inner.inner.len,
            &mut self.inner.inner.inner.stride,
        );
    }
    pub fn transpose13(&mut self) {
        self.inner.transpose02();
    }
    pub fn transpose23(&mut self) {
        self.inner.inner.transpose01();
    }
}

pub struct Strided<T, P = <T as FromPtr>::Pointee, S = <T as FromPtr>::State>
where
    T: FromPtr<Pointee = P, State = S>,
{
    ptr: NonNull<P>,
    state: StridedState<S>,
    _marker: PhantomData<T>,
}

impl<T: FromPtr<State = ()>> Strided<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn from_slice<U>(mut slice: U) -> Self
    where
        U: SliceLike<GetItem = T>,
        Self: From<U>,
    {
        unsafe {
            Self::from_raw_parts(
                slice.as_non_null_ptr(),
                StridedState {
                    len: slice.len(),
                    stride: slice.stride(),
                    inner: (),
                },
            )
        }
    }
}

impl<T: FromPtr> Strided<T> {
    pub unsafe fn from_raw_parts(ptr: NonNull<T::Pointee>, state: StridedState<T::State>) -> Self {
        Strided {
            ptr,
            state,
            _marker: PhantomData,
        }
    }

    pub unsafe fn cast_as<U>(&self) -> Strided<U>
    where
        U: FromPtr<State = T::State>,
    {
        unsafe { Strided::from_raw_parts(self.ptr.cast(), self.state.clone()) }
    }

    pub unsafe fn cast_into<U>(self) -> Strided<U>
    where
        U: FromPtr<State = T::State>,
    {
        unsafe { self.cast_as() }
    }

    pub fn as_non_null_ptr(&self) -> NonNull<T::Pointee> {
        self.ptr
    }
    pub fn as_ptr(&self) -> *mut T::Pointee {
        self.ptr.as_ptr()
    }
    pub fn state(&self) -> &StridedState<T::State> {
        &self.state
    }
    pub fn len(&self) -> usize {
        self.state.len
    }
    pub fn stride(&self) -> isize {
        self.state.stride
    }
    pub fn total_size(&self) -> usize {
        <Self as FromPtr>::state_total_size(&self.state)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub unsafe fn unchecked_into_get(self, i: usize) -> T {
        unsafe {
            let offset = self.stride().unchecked_mul(i as isize);
            let ptr = self.as_non_null_ptr().byte_offset(offset);
            T::from_ptr(ptr, self.state().inner.clone())
        }
    }
    pub fn checked_into_get(self, i: usize) -> Option<T> {
        if i < self.len() {
            Some(unsafe { self.unchecked_into_get(i) })
        } else {
            None
        }
    }
    pub fn into_get(self, i: usize) -> T {
        self.checked_into_get(i).expect("Index out of bound")
    }
    unsafe fn unchecked_clone(&self) -> Self {
        unsafe { self.cast_as() }
    }

    pub fn into_split_first(self) -> Option<(T, Strided<T>)> {
        match self.len() {
            0 => None,
            1 => unsafe {
                Some((
                    self.unchecked_clone().unchecked_into_get(0),
                    self.unchecked_into_partial(0, 0, 0),
                ))
            },
            len => unsafe {
                Some((
                    self.unchecked_clone().unchecked_into_get(0),
                    self.unchecked_into_partial(1, len.unchecked_sub(1), 1),
                ))
            },
        }
    }
    pub fn into_split_last(self) -> Option<(T, Strided<T>)> {
        if !self.is_empty() {
            unsafe {
                let len = self.len().unchecked_sub(1);
                Some((
                    self.unchecked_clone().unchecked_into_get(len),
                    self.unchecked_into_partial(0, len, 1),
                ))
            }
        } else {
            None
        }
    }

    pub unsafe fn unchecked_into_split_at(self, i: usize) -> (Strided<T>, Strided<T>) {
        unsafe {
            let len = self.len().unchecked_sub(i);
            (
                self.unchecked_clone().unchecked_into_partial(0, i, 1),
                self.unchecked_into_partial(i, len, 1),
            )
        }
    }
    pub fn checked_into_split_at(self, i: usize) -> Option<(Strided<T>, Strided<T>)> {
        match i.cmp(&self.len()) {
            std::cmp::Ordering::Less => Some(unsafe { self.unchecked_into_split_at(i) }),
            std::cmp::Ordering::Equal => {
                Some(unsafe { (self.unchecked_clone(), self.unchecked_into_partial(0, 0, 0)) })
            }
            std::cmp::Ordering::Greater => None,
        }
    }
    pub fn into_split_at(self, i: usize) -> (Strided<T>, Strided<T>) {
        self.checked_into_split_at(i).expect("Split out of bound")
    }

    pub unsafe fn unchecked_into_partial(
        self,
        start: usize,
        len: usize,
        step_by: isize,
    ) -> Strided<T> {
        unsafe {
            let i = if step_by.is_positive() {
                start
            } else {
                self.len().saturating_sub(1)
            };

            // SAFETY: i is in 0..self.len
            let offset = self.stride().unchecked_mul(i as isize);
            let ptr = self.ptr.byte_offset(offset);

            let stride = step_by.saturating_mul(self.stride());
            Self::from_raw_parts(
                ptr,
                StridedState {
                    len,
                    stride,
                    inner: self.state.inner.clone(),
                },
            )
        }
    }

    pub fn into_partial(self, start: usize, len: usize, step_by: NonZero<isize>) -> Strided<T> {
        if start < self.len() {
            unsafe {
                let mut available_len = self.len().saturating_sub(start);
                if available_len > 0 {
                    available_len = available_len
                        .unchecked_sub(1)
                        .div_euclid(step_by.get().unsigned_abs())
                        .unchecked_add(1);
                }
                let len = len.min(available_len);

                self.unchecked_into_partial(start, len, step_by.get())
            }
        } else {
            unsafe { self.unchecked_into_partial(0, 0, 0) }
        }
    }

    pub unsafe fn unchecked_into_chunks(self, n: NonZero<usize>) -> Strided<Strided<T>> {
        unsafe {
            let outer_len = self.len() / n;
            let inner_len = n.get();
            Strided::from_raw_parts(
                self.ptr,
                StridedState {
                    len: outer_len,
                    stride: self.stride().unchecked_mul(inner_len as isize),
                    inner: StridedState {
                        len: inner_len,
                        stride: self.stride(),
                        inner: self.state.inner,
                    },
                },
            )
        }
    }
    pub fn into_chunks(self, n: NonZero<usize>) -> (Strided<Strided<T>>, Strided<T>) {
        let len = self.len();
        let rem = len % n;
        let (head, tail) = if rem == 0 {
            unsafe { (self.unchecked_clone(), self.unchecked_into_partial(0, 0, 0)) }
        } else {
            unsafe { self.unchecked_into_split_at(len.unchecked_sub(rem)) }
        };
        (unsafe { head.unchecked_into_chunks(n) }, tail)
    }
    pub fn into_rchunks(self, n: NonZero<usize>) -> (Strided<T>, Strided<Strided<T>>) {
        let len = self.len();
        let rem = len % n;
        unsafe {
            let (head, tail) = self.unchecked_into_split_at(rem);
            (head, tail.unchecked_into_chunks(n))
        }
    }

    pub fn into_deinterleave_array<const N: usize>(self) -> [Strided<T>; N] {
        match NonZero::new(N as isize) {
            Some(n) => std::array::from_fn(|i| unsafe {
                self.unchecked_clone().into_partial(i, self.len(), n)
            }),
            None => panic!("N should be non zero"),
        }
    }
}

impl<T: FromPtr> Strided<T> {
    pub fn borrow(&self) -> Strided<T::Borrow<'_>> {
        unsafe { self.cast_as() }
    }
    pub fn into_borrow<'a>(self) -> Strided<T::Borrow<'a>>
    where
        Self: 'a,
    {
        unsafe { self.cast_as() }
    }
    pub fn iter(&self) -> Iter<T::Borrow<'_>> {
        Iter(self.borrow())
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> T::Borrow<'_> {
        unsafe { self.borrow().unchecked_into_get(i) }
    }
    pub fn checked_get(&self, i: usize) -> Option<T::Borrow<'_>> {
        self.borrow().checked_into_get(i)
    }
    pub fn get(&self, i: usize) -> T::Borrow<'_> {
        self.borrow().into_get(i)
    }

    pub fn first(&self) -> Option<T::Borrow<'_>> {
        self.checked_get(0)
    }
    pub fn last(&self) -> Option<T::Borrow<'_>> {
        self.checked_get(self.len().wrapping_sub(1))
    }

    pub fn split_first(&self) -> Option<(T::Borrow<'_>, Strided<T::Borrow<'_>>)> {
        self.borrow().into_split_first()
    }
    pub fn split_last(&self) -> Option<(T::Borrow<'_>, Strided<T::Borrow<'_>>)> {
        self.borrow().into_split_last()
    }
    pub unsafe fn unchecked_split_at(
        &self,
        i: usize,
    ) -> (Strided<T::Borrow<'_>>, Strided<T::Borrow<'_>>) {
        unsafe { self.borrow().unchecked_into_split_at(i) }
    }
    #[allow(clippy::type_complexity)]
    pub fn checked_split_at(
        &self,
        i: usize,
    ) -> Option<(Strided<T::Borrow<'_>>, Strided<T::Borrow<'_>>)> {
        self.borrow().checked_into_split_at(i)
    }
    pub fn split_at(&self, i: usize) -> (Strided<T::Borrow<'_>>, Strided<T::Borrow<'_>>) {
        self.borrow().into_split_at(i)
    }

    pub unsafe fn unchecked_partial(
        &self,
        start: usize,
        len: usize,
        step_by: isize,
    ) -> Strided<T::Borrow<'_>> {
        unsafe { self.borrow().unchecked_into_partial(start, len, step_by) }
    }

    pub fn partial(
        &self,
        start: usize,
        len: usize,
        step_by: NonZero<isize>,
    ) -> Strided<T::Borrow<'_>> {
        self.borrow().into_partial(start, len, step_by)
    }

    pub unsafe fn unchecked_as_chunks(&self, n: NonZero<usize>) -> Strided<Strided<T::Borrow<'_>>> {
        unsafe { self.borrow().unchecked_into_chunks(n) }
    }
    pub fn as_chunks(
        &self,
        n: NonZero<usize>,
    ) -> (Strided<Strided<T::Borrow<'_>>>, Strided<T::Borrow<'_>>) {
        self.borrow().into_chunks(n)
    }
    pub fn as_rchunks(
        &self,
        n: NonZero<usize>,
    ) -> (Strided<T::Borrow<'_>>, Strided<Strided<T::Borrow<'_>>>) {
        self.borrow().into_rchunks(n)
    }

    pub fn deinterleave_array<const N: usize>(&self) -> [Strided<T::Borrow<'_>>; N] {
        self.borrow().into_deinterleave_array()
    }
}

impl<T: FromPtrMut> Strided<T> {
    pub fn borrow_mut(&mut self) -> Strided<T::BorrowMut<'_>> {
        unsafe { self.cast_as() }
    }
    pub fn into_borrow_mut<'a>(self) -> Strided<T::BorrowMut<'a>>
    where
        Self: 'a,
    {
        unsafe { self.cast_as() }
    }

    pub fn iter_mut(&'_ mut self) -> Iter<T::BorrowMut<'_>> {
        Iter(self.borrow_mut())
    }

    pub unsafe fn unchecked_get_mut(&mut self, i: usize) -> T::BorrowMut<'_> {
        unsafe { self.borrow_mut().unchecked_into_get(i) }
    }
    pub fn checked_get_mut(&mut self, i: usize) -> Option<T::BorrowMut<'_>> {
        self.borrow_mut().checked_into_get(i)
    }
    pub fn get_mut(&mut self, i: usize) -> T::BorrowMut<'_> {
        self.borrow_mut().into_get(i)
    }

    pub fn first_mut(&mut self) -> Option<T::BorrowMut<'_>> {
        self.checked_get_mut(0)
    }
    pub fn last_mut(&mut self) -> Option<T::BorrowMut<'_>> {
        self.checked_get_mut(self.len().wrapping_sub(1))
    }

    pub fn split_first_mut(&mut self) -> Option<(T::BorrowMut<'_>, Strided<T::BorrowMut<'_>>)> {
        self.borrow_mut().into_split_first()
    }
    pub fn split_last_mut(&mut self) -> Option<(T::BorrowMut<'_>, Strided<T::BorrowMut<'_>>)> {
        self.borrow_mut().into_split_last()
    }
    pub unsafe fn unchecked_split_at_mut(
        &mut self,
        i: usize,
    ) -> (Strided<T::BorrowMut<'_>>, Strided<T::BorrowMut<'_>>) {
        unsafe { self.borrow_mut().unchecked_into_split_at(i) }
    }
    #[allow(clippy::type_complexity)]
    pub fn checked_split_at_mut(
        &mut self,
        i: usize,
    ) -> Option<(Strided<T::BorrowMut<'_>>, Strided<T::BorrowMut<'_>>)> {
        self.borrow_mut().checked_into_split_at(i)
    }
    pub fn split_at_mut(
        &mut self,
        i: usize,
    ) -> (Strided<T::BorrowMut<'_>>, Strided<T::BorrowMut<'_>>) {
        self.borrow_mut().into_split_at(i)
    }

    pub unsafe fn unchecked_partial_mut(
        &mut self,
        start: usize,
        len: usize,
        step_by: isize,
    ) -> Strided<T::BorrowMut<'_>> {
        unsafe {
            self.borrow_mut()
                .unchecked_into_partial(start, len, step_by)
        }
    }

    pub fn partial_mut(
        &mut self,
        start: usize,
        len: usize,
        step_by: NonZero<isize>,
    ) -> Strided<T::BorrowMut<'_>> {
        self.borrow_mut().into_partial(start, len, step_by)
    }
    pub unsafe fn unchecked_as_chunks_mut(
        &mut self,
        n: NonZero<usize>,
    ) -> Strided<Strided<T::BorrowMut<'_>>> {
        unsafe { self.borrow_mut().unchecked_into_chunks(n) }
    }
    pub fn as_chunks_mut(
        &mut self,
        n: NonZero<usize>,
    ) -> (
        Strided<Strided<T::BorrowMut<'_>>>,
        Strided<T::BorrowMut<'_>>,
    ) {
        self.borrow_mut().into_chunks(n)
    }
    pub fn as_rchunks_mut(
        &mut self,
        n: NonZero<usize>,
    ) -> (
        Strided<T::BorrowMut<'_>>,
        Strided<Strided<T::BorrowMut<'_>>>,
    ) {
        self.borrow_mut().into_rchunks(n)
    }

    pub fn deinterleave_array_mut<const N: usize>(&mut self) -> [Strided<T::BorrowMut<'_>>; N] {
        self.borrow_mut().into_deinterleave_array()
    }
}

impl<'a, T: RefLike<'a>> Strided<T> {
    pub fn as_strided_ref(&self) -> Strided<T::Borrow<'_>> {
        self.borrow()
    }
    pub fn into_strided_ref(self) -> Strided<T::Borrow<'a>> {
        self.into_borrow()
    }
    pub fn as_strided_ptr(&self) -> Strided<T::AsPtr> {
        unsafe { self.cast_as() }
    }
}

impl<'a, T: MutLike<'a>> Strided<T> {
    pub fn as_strided_mut(&mut self) -> Strided<T::BorrowMut<'_>> {
        self.borrow_mut()
    }
    pub fn into_strided_mut(self) -> Strided<T::BorrowMut<'a>> {
        self.into_borrow_mut()
    }
}

impl<T: FromPtr + PtrLike> Strided<T> {
    pub unsafe fn cast_as_ref<'a>(&self) -> Strided<T::AsRef<'a>>
    where
        T: 'a,
    {
        unsafe { self.cast_as() }
    }
    pub unsafe fn cast_as_mut<'a>(&self) -> Strided<T::AsMut<'a>>
    where
        T: 'a,
    {
        unsafe { self.cast_as() }
    }
}

pub struct StridedBlocks<T: FromPtr> {
    pub blocks: Strided<Strided<T>>,
    pub remaining0: Strided<T>,
    pub remaining1: Strided<T>,
    pub remaining01: T,
}

impl<T: FromPtr> Strided<Strided<T>> {
    pub fn from_matrix<U>(matrix: U) -> Self
    where
        U: GridLike<CellItem = T>,
        U::GetItem: FromPtr<State = ()>,
        Strided<U::GetItem>: From<U>,
        Self: From<Strided<U::GetItem>>,
    {
        Strided::from_slice(matrix).into()
    }
    pub fn transpose01(&mut self) -> &mut Strided<Strided<T>> {
        self.state.transpose01();
        self
    }
    pub fn into_transpose01(mut self) -> Strided<Strided<T>> {
        self.transpose01();
        self
    }

    pub unsafe fn unchecked_into_blocks(
        mut self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> Strided<Strided<Strided<Strided<T>>>> {
        // ij -> ji
        self.transpose01();
        // ji -> Jji
        let mut slices = unsafe { self.unchecked_into_chunks(n1) };
        // Jji -> Jij
        slices.transpose12();
        // Jij -> iJj
        slices.transpose01();
        // iJj -> IiJj
        let mut blocks = unsafe { slices.unchecked_into_chunks(n0) };
        // IiJj -> IJij
        blocks.transpose12();

        blocks
    }

    pub fn into_blocks(
        mut self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> StridedBlocks<Strided<Strided<T>>> {
        // ij -> ji
        self.transpose01();

        // ji -> Jji
        let (mut slices, mut remaining1) = self.into_chunks(n1);

        // Jji -> Jij
        slices.transpose12();
        // Jij -> iJj
        slices.transpose01();
        // remaining1 ji -> ij
        remaining1.transpose01();

        // remaining1 ij -> Iij
        let (remaining1, remaining01) = remaining1.into_chunks(n0);

        // iJj -> IiJj
        let (mut blocks, mut remaining0) = slices.into_chunks(n0);

        // IiJj -> IJij
        blocks.transpose12();
        // remaining0 iJj -> Jij
        remaining0.transpose01();

        StridedBlocks {
            blocks,
            remaining0,
            remaining1,
            remaining01,
        }
    }
}

impl<T: FromPtr> Strided<Strided<T>> {
    pub fn as_transpose01(&self) -> Strided<Strided<T::Borrow<'_>>> {
        self.borrow().into_transpose01()
    }

    pub unsafe fn unchecked_as_blocks(
        &self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> Strided<Strided<Strided<Strided<T::Borrow<'_>>>>> {
        unsafe { self.borrow().unchecked_into_blocks(n0, n1) }
    }

    pub fn as_blocks(
        &self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> StridedBlocks<Strided<Strided<T::Borrow<'_>>>> {
        self.borrow().into_blocks(n0, n1)
    }
}
impl<T: FromPtrMut> Strided<Strided<T>> {
    pub fn as_transpose01_mut(&mut self) -> Strided<Strided<T::BorrowMut<'_>>> {
        self.borrow_mut().into_transpose01()
    }

    pub unsafe fn unchecked_as_blocks_mut(
        &mut self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> Strided<Strided<Strided<Strided<T::BorrowMut<'_>>>>> {
        unsafe { self.borrow_mut().unchecked_into_blocks(n0, n1) }
    }

    pub fn as_blocks_mut(
        &mut self,
        n0: NonZero<usize>,
        n1: NonZero<usize>,
    ) -> StridedBlocks<Strided<Strided<T::BorrowMut<'_>>>> {
        self.borrow_mut().into_blocks(n0, n1)
    }
}

impl<T: FromPtr> Strided<Strided<Strided<T>>> {
    pub fn transpose02(&mut self) -> &mut Strided<Strided<Strided<T>>> {
        self.state.transpose02();
        self
    }
    pub fn into_transpose02(mut self) -> Strided<Strided<Strided<T>>> {
        self.transpose02();
        self
    }

    pub fn transpose12(&mut self) -> &mut Strided<Strided<Strided<T>>> {
        self.state.transpose12();
        self
    }
    pub fn into_transpose12(mut self) -> Strided<Strided<Strided<T>>> {
        self.transpose12();
        self
    }
}
impl<T: FromPtr> Strided<Strided<Strided<T>>> {
    pub fn as_transpose02(&self) -> Strided<Strided<Strided<T::Borrow<'_>>>> {
        self.borrow().into_transpose02()
    }
    pub fn as_transpose12(&self) -> Strided<Strided<Strided<T::Borrow<'_>>>> {
        self.borrow().into_transpose12()
    }
}
impl<T: FromPtrMut> Strided<Strided<Strided<T>>> {
    pub fn as_transpose02_mut(&mut self) -> Strided<Strided<Strided<T::BorrowMut<'_>>>> {
        self.borrow_mut().into_transpose02()
    }
    pub fn as_transpose12_mut(&mut self) -> Strided<Strided<Strided<T::BorrowMut<'_>>>> {
        self.borrow_mut().into_transpose12()
    }
}

impl<T: FromPtr> Strided<Strided<Strided<Strided<T>>>> {
    pub fn transpose03(&mut self) -> &mut Strided<Strided<Strided<Strided<T>>>> {
        self.state.transpose03();
        self
    }
    pub fn into_transpose03(mut self) -> Strided<Strided<Strided<Strided<T>>>> {
        self.transpose03();
        self
    }

    pub fn transpose13(&mut self) -> &mut Strided<Strided<Strided<Strided<T>>>> {
        self.state.transpose13();
        self
    }
    pub fn into_transpose13(mut self) -> Strided<Strided<Strided<Strided<T>>>> {
        self.transpose13();
        self
    }

    pub fn transpose23(&mut self) -> &mut Strided<Strided<Strided<Strided<T>>>> {
        self.state.transpose23();
        self
    }
    pub fn into_transpose23(mut self) -> Strided<Strided<Strided<Strided<T>>>> {
        self.transpose23();
        self
    }
}
impl<T: FromPtr> Strided<Strided<Strided<Strided<T>>>> {
    pub fn as_transpose03(&self) -> Strided<Strided<Strided<Strided<T::Borrow<'_>>>>> {
        self.borrow().into_transpose03()
    }
    pub fn as_transpose13(&self) -> Strided<Strided<Strided<Strided<T::Borrow<'_>>>>> {
        self.borrow().into_transpose13()
    }
    pub fn as_transpose23(&self) -> Strided<Strided<Strided<Strided<T::Borrow<'_>>>>> {
        self.borrow().into_transpose23()
    }
}
impl<T: FromPtrMut> Strided<Strided<Strided<Strided<T>>>> {
    pub fn as_transpose03_mut(&mut self) -> Strided<Strided<Strided<Strided<T::BorrowMut<'_>>>>> {
        self.borrow_mut().into_transpose03()
    }
    pub fn as_transpose13_mut(&mut self) -> Strided<Strided<Strided<Strided<T::BorrowMut<'_>>>>> {
        self.borrow_mut().into_transpose13()
    }
    pub fn as_transpose23_mut(&mut self) -> Strided<Strided<Strided<Strided<T::BorrowMut<'_>>>>> {
        self.borrow_mut().into_transpose23()
    }
}

impl<T: FromPtr> Default for Strided<T> {
    fn default() -> Self {
        unsafe { Self::from_raw_parts(NonNull::dangling(), Default::default()) }
    }
}

impl<T: FromPtr + Clone> Clone for Strided<T> {
    fn clone(&self) -> Self {
        unsafe { self.cast_as() }
    }
}
impl<T> Copy for Strided<T>
where
    T: FromPtr + Copy,
    T::State: Copy,
{
}

unsafe impl<T: FromPtr> Send for Strided<T>
where
    T: Send,
    T::State: Send,
{
}
unsafe impl<T: FromPtr> Sync for Strided<T>
where
    T: Sync,
    T::State: Sync,
{
}

impl<T: FromPtr<State = ()>, U: SliceLike<GetItem = T>> From<U> for Strided<T> {
    fn from(value: U) -> Self {
        Self::from_slice(value)
    }
}

impl<'a, T> From<&'a [T]> for Strided<NonNull<T>> {
    fn from(value: &'a [T]) -> Self {
        Self::from(<NonNull<[T]>>::from(value))
    }
}
impl<'a, T> From<&'a mut [T]> for Strided<NonNull<T>> {
    fn from(value: &'a mut [T]) -> Self {
        Self::from(<NonNull<[T]>>::from(value))
    }
}
impl<'a, T> From<&'a mut [T]> for Strided<&'a T> {
    fn from(value: &'a mut [T]) -> Self {
        Self::from(<&[T]>::from(value))
    }
}
impl<'a, T> From<Strided<&'a mut T>> for Strided<&'a T> {
    fn from(value: Strided<&'a mut T>) -> Self {
        value.into_strided_ref()
    }
}

impl<T> From<Strided<NonNull<[T]>>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<NonNull<[T]>>) -> Self {
        unsafe {
            Strided::from_raw_parts(
                value.as_non_null_ptr().cast(),
                StridedState {
                    len: value.len(),
                    stride: value.stride(),
                    inner: StridedState {
                        len: value.state().inner,
                        stride: std::mem::size_of::<T>() as isize,
                        inner: (),
                    },
                },
            )
        }
    }
}
impl<T, const N: usize> From<Strided<NonNull<[T; N]>>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<NonNull<[T; N]>>) -> Self {
        match NonZero::new(N) {
            Some(n) => unsafe {
                Strided::from_raw_parts(
                    value.as_non_null_ptr().cast(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: StridedState {
                            len: n.get(),
                            stride: std::mem::size_of::<T>() as isize,
                            inner: (),
                        },
                    },
                )
            },
            None => unsafe {
                Strided::from_raw_parts(
                    value.as_non_null_ptr().cast(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: StridedState {
                            len: 0,
                            stride: 0,
                            inner: (),
                        },
                    },
                )
            },
        }
    }
}

impl<'a, T> From<Strided<&'a [T]>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<&'a [T]>) -> Self {
        Self::from(value.as_strided_ptr())
    }
}
impl<'a, T, const N: usize> From<Strided<&'a [T; N]>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<&'a [T; N]>) -> Self {
        Self::from(value.as_strided_ptr())
    }
}
impl<'a, T> From<Strided<&'a mut [T]>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<&'a mut [T]>) -> Self {
        Self::from(value.as_strided_ptr())
    }
}
impl<'a, T, const N: usize> From<Strided<&'a mut [T; N]>> for Strided<Strided<NonNull<T>>> {
    fn from(value: Strided<&'a mut [T; N]>) -> Self {
        Self::from(value.as_strided_ptr())
    }
}

impl<'a, T, const N: usize> From<&'a [[T; N]]> for Strided<Strided<NonNull<T>>> {
    fn from(value: &'a [[T; N]]) -> Self {
        Self::from(Strided::<NonNull<[T; N]>>::from(value))
    }
}
impl<'a, T, const N: usize> From<&'a mut [[T; N]]> for Strided<Strided<NonNull<T>>> {
    fn from(value: &'a mut [[T; N]]) -> Self {
        <Self as From<&'a [[T; N]]>>::from(value)
    }
}

impl<'a, T> From<Strided<&'a [T]>> for Strided<Strided<&'a T>> {
    fn from(value: Strided<&'a [T]>) -> Self {
        unsafe { Strided::<Strided<NonNull<T>>>::from(value).cast_as_ref() }
    }
}
impl<'a, T, const N: usize> From<Strided<&'a [T; N]>> for Strided<Strided<&'a T>> {
    fn from(value: Strided<&'a [T; N]>) -> Self {
        unsafe { Strided::<Strided<NonNull<T>>>::from(value).cast_as_ref() }
    }
}
impl<'a, T> From<Strided<&'a mut [T]>> for Strided<Strided<&'a T>> {
    fn from(value: Strided<&'a mut [T]>) -> Self {
        Self::from(value.into_strided_ref())
    }
}
impl<'a, T, const N: usize> From<Strided<&'a mut [T; N]>> for Strided<Strided<&'a T>> {
    fn from(value: Strided<&'a mut [T; N]>) -> Self {
        Self::from(value.into_strided_ref())
    }
}

impl<'a, T, const N: usize> From<&'a [[T; N]]> for Strided<Strided<&'a T>> {
    fn from(value: &'a [[T; N]]) -> Self {
        Self::from(Strided::<&'a [T; N]>::from(value))
    }
}
impl<'a, T, const N: usize> From<&'a mut [[T; N]]> for Strided<Strided<&'a T>> {
    fn from(value: &'a mut [[T; N]]) -> Self {
        Self::from(Strided::<&'a [T; N]>::from(value))
    }
}

impl<'a, T> From<Strided<&'a mut [T]>> for Strided<Strided<&'a mut T>> {
    fn from(value: Strided<&'a mut [T]>) -> Self {
        unsafe { Strided::<Strided<NonNull<T>>>::from(value).cast_as_mut() }
    }
}
impl<'a, T, const N: usize> From<Strided<&'a mut [T; N]>> for Strided<Strided<&'a mut T>> {
    fn from(value: Strided<&'a mut [T; N]>) -> Self {
        unsafe { Strided::<Strided<NonNull<T>>>::from(value).cast_as_mut() }
    }
}
impl<'a, T, const N: usize> From<&'a mut [[T; N]]> for Strided<Strided<&'a mut T>> {
    fn from(value: &'a mut [[T; N]]) -> Self {
        Self::from(Strided::<&'a mut [T; N]>::from(value))
    }
}

impl<'a, T: FromPtr, U: FromPtr> From<&'a Strided<T>> for Strided<U>
where
    Strided<U>: From<Strided<T::Borrow<'a>>>,
{
    fn from(value: &'a Strided<T>) -> Self {
        value.borrow().into()
    }
}
impl<'a, T: FromPtrMut, U: FromPtr> From<&'a mut Strided<T>> for Strided<U>
where
    Strided<U>: From<Strided<T::BorrowMut<'a>>>,
{
    fn from(value: &'a mut Strided<T>) -> Self {
        value.borrow_mut().into()
    }
}
impl<T> TryFrom<Strided<NonNull<T>>> for NonNull<[T]> {
    type Error = Strided<NonNull<T>>;

    fn try_from(value: Strided<NonNull<T>>) -> Result<Self, Self::Error> {
        if value.stride() as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                <NonNull<[T]> as FromPtr>::from_ptr(value.as_non_null_ptr(), value.state().len)
            })
        } else {
            Err(value)
        }
    }
}
impl<'a, T> TryFrom<Strided<&'a T>> for &'a [T] {
    type Error = Strided<&'a T>;

    fn try_from(value: Strided<&'a T>) -> Result<Self, Self::Error> {
        if value.stride() as usize == std::mem::size_of::<T>() {
            Ok(unsafe { <&[T] as FromPtr>::from_ptr(value.as_non_null_ptr(), value.state().len) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T> TryFrom<Strided<&'a mut T>> for &'a mut [T] {
    type Error = Strided<&'a mut T>;

    fn try_from(value: Strided<&'a mut T>) -> Result<Self, Self::Error> {
        if value.stride() as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                <&mut [T] as FromPtr>::from_ptr(value.as_non_null_ptr(), value.state().len)
            })
        } else {
            Err(value)
        }
    }
}
impl<'a, T> TryFrom<Strided<&'a mut T>> for &'a [T] {
    type Error = Strided<&'a mut T>;

    fn try_from(value: Strided<&'a mut T>) -> Result<Self, Self::Error> {
        if value.stride() as usize == std::mem::size_of::<T>() {
            Ok(unsafe { <&[T] as FromPtr>::from_ptr(value.as_non_null_ptr(), value.state().len) })
        } else {
            Err(value)
        }
    }
}

impl<T> TryFrom<Strided<Strided<NonNull<T>>>> for Strided<NonNull<[T]>> {
    type Error = Strided<Strided<NonNull<T>>>;

    fn try_from(value: Strided<Strided<NonNull<T>>>) -> Result<Self, Self::Error> {
        if value.state.inner.stride as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                Self::from_raw_parts(
                    value.as_non_null_ptr(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: value.state.inner.len,
                    },
                )
            })
        } else {
            Err(value)
        }
    }
}
impl<'a, T> TryFrom<Strided<Strided<&'a T>>> for Strided<&'a [T]> {
    type Error = Strided<Strided<&'a T>>;

    fn try_from(value: Strided<Strided<&'a T>>) -> Result<Self, Self::Error> {
        if value.state.inner.stride as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                Self::from_raw_parts(
                    value.as_non_null_ptr(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: value.state.inner.len,
                    },
                )
            })
        } else {
            Err(value)
        }
    }
}

impl<'a, T> TryFrom<Strided<Strided<&'a mut T>>> for Strided<&'a mut [T]> {
    type Error = Strided<Strided<&'a mut T>>;

    fn try_from(value: Strided<Strided<&'a mut T>>) -> Result<Self, Self::Error> {
        if value.state.inner.stride as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                Self::from_raw_parts(
                    value.as_non_null_ptr(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: value.state.inner.len,
                    },
                )
            })
        } else {
            Err(value)
        }
    }
}
impl<'a, T> TryFrom<Strided<Strided<&'a mut T>>> for Strided<&'a [T]> {
    type Error = Strided<Strided<&'a mut T>>;

    fn try_from(value: Strided<Strided<&'a mut T>>) -> Result<Self, Self::Error> {
        if value.state.inner.stride as usize == std::mem::size_of::<T>() {
            Ok(unsafe {
                Self::from_raw_parts(
                    value.as_non_null_ptr(),
                    StridedState {
                        len: value.len(),
                        stride: value.stride(),
                        inner: value.state.inner.len,
                    },
                )
            })
        } else {
            Err(value)
        }
    }
}

impl<'a, 'b, T, U> PartialEq<Strided<U>> for Strided<T>
where
    T: FromPtr + PartialEq<U> + 'a,
    U: FromPtr + 'b,
    T::Borrow<'a>: PartialEq<U::Borrow<'b>>,
{
    fn eq(&self, other: &Strided<U>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        // SAFETY:
        // It is most likely unsound considering interior mutability.
        // The problem is the cast to a broader lifetime that one can get in the implementation of the PartialEq trait,
        // And which is uncorrelated from a compiler point of view to the lifetime of &self or &other.
        // The bound T: PartialEq<U> might be enough to make it sound because it is not possible to talk about
        //   this broader lifetime in the PartialEq implementation.
        // Even if this makes it sound, this would break when specialization becomes stable.
        let mut iter_a = unsafe { self.cast_as::<T::Borrow<'a>>().into_iter() };
        let mut iter_b = unsafe { other.cast_as::<U::Borrow<'b>>().into_iter() };

        loop {
            match (iter_a.next(), iter_b.next()) {
                (None, None) => return true,
                (None, Some(_)) => return false,
                (Some(_), None) => return false,
                (Some(a), Some(b)) => {
                    if a != b {
                        return false;
                    }
                }
            }
        }
    }
}
impl<'a, 'b, T, U> PartialEq<&'b [U]> for Strided<T>
where
    T: FromPtr + 'a,
    T: PartialEq<&'b U>,
    T::Borrow<'a>: PartialEq<&'b U>,
{
    fn eq(&self, other: &&'b [U]) -> bool {
        self.eq(&Strided::from_slice(*other))
    }
}
impl<'a, T, U> PartialEq<[U]> for Strided<T>
where
    T: FromPtr + 'a,
    T: for<'b> PartialEq<&'b U>,
    T::Borrow<'a>: for<'b> PartialEq<&'b U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.eq(&other)
    }
}
impl<'a, 'b, T, U, const N: usize> PartialEq<&'b [U; N]> for Strided<T>
where
    T: FromPtr + 'a,
    T: PartialEq<&'b U>,
    T::Borrow<'a>: PartialEq<&'b U>,
{
    fn eq(&self, other: &&'b [U; N]) -> bool {
        self.eq(&Strided::from_slice(*other))
    }
}
impl<'a, T, U, const N: usize> PartialEq<[U; N]> for Strided<T>
where
    T: FromPtr + 'a,
    T: for<'b> PartialEq<&'b U>,
    T::Borrow<'a>: for<'b> PartialEq<&'b U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.eq(&other)
    }
}
impl<'a, 'b, T, U> PartialEq<Strided<U>> for &'b [T]
where
    U: FromPtr + 'a,
    &'b T: PartialEq<U> + PartialEq<U::Borrow<'a>>,
{
    fn eq(&self, other: &Strided<U>) -> bool {
        Strided::from_slice(*self).eq(other)
    }
}
impl<'a, T, U> PartialEq<Strided<U>> for [T]
where
    U: FromPtr + 'a,
    for<'b> &'b T: PartialEq<U> + PartialEq<U::Borrow<'a>>,
{
    fn eq(&self, other: &Strided<U>) -> bool {
        Strided::from_slice(self).eq(other)
    }
}
impl<'a, T, U, const N: usize> PartialEq<Strided<U>> for [T; N]
where
    U: FromPtr + 'a,
    for<'b> &'b T: PartialEq<U> + PartialEq<U::Borrow<'a>>,
{
    fn eq(&self, other: &Strided<U>) -> bool {
        Strided::from_slice(self.as_slice()).eq(other)
    }
}
impl<'a, 'b, T, U, const N: usize> PartialEq<Strided<U>> for &'b [T; N]
where
    U: FromPtr + 'a,
    &'b T: PartialEq<U> + PartialEq<U::Borrow<'a>>,
{
    fn eq(&self, other: &Strided<U>) -> bool {
        Strided::from_slice(self.as_slice()).eq(other)
    }
}

impl<'a, T> Eq for Strided<T>
where
    T: FromPtr + Eq + 'a,
    T::Borrow<'a>: Eq,
{
}

impl<'a, 'b, T, U> PartialOrd<Strided<U>> for Strided<T>
where
    T: FromPtr + PartialOrd<U> + 'a,
    U: FromPtr + 'b,
    T::Borrow<'a>: PartialOrd<U::Borrow<'b>>,
{
    fn partial_cmp(&self, other: &Strided<U>) -> Option<std::cmp::Ordering> {
        // SAFETY:
        // It is most likely unsound considering interior mutability.
        // The problem is the cast to a broader lifetime that one can get in the implementation of the PartialOrd trait,
        // And which is uncorrelated from a compiler point of view to the lifetime of &self or &other.
        // The bound T: PartialOrd<U> might be enough to make it sound because it is not possible to talk about
        //   this broader lifetime in the PartialOrd implementation.
        // Even if this makes it sound, this would break when specialization becomes stable.
        let mut iter_a = unsafe { self.cast_as::<T::Borrow<'a>>().into_iter() };
        let mut iter_b = unsafe { other.cast_as::<U::Borrow<'b>>().into_iter() };

        loop {
            match (iter_a.next(), iter_b.next()) {
                (None, None) => return Some(std::cmp::Ordering::Equal),
                (None, Some(_)) => return Some(std::cmp::Ordering::Greater),
                (Some(_), None) => return Some(std::cmp::Ordering::Less),
                (Some(a), Some(b)) => match a.partial_cmp(&b) {
                    Some(std::cmp::Ordering::Less) => return Some(std::cmp::Ordering::Less),
                    Some(std::cmp::Ordering::Equal) => (),
                    Some(std::cmp::Ordering::Greater) => return Some(std::cmp::Ordering::Greater),
                    None => return None,
                },
            }
        }
    }
}

impl<'a, 'b, T, U> PartialOrd<&'b [U]> for Strided<T>
where
    T: FromPtr + 'a,
    T: PartialOrd<&'b U>,
    T::Borrow<'a>: PartialOrd<&'b U>,
{
    fn partial_cmp(&self, other: &&'b [U]) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Strided::from_slice(*other))
    }
}
impl<'a, T, U> PartialOrd<[U]> for Strided<T>
where
    T: FromPtr + 'a,
    T: for<'b> PartialOrd<&'b U>,
    T::Borrow<'a>: for<'b> PartialOrd<&'b U>,
{
    fn partial_cmp(&self, other: &[U]) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other)
    }
}
impl<'a, 'b, T, U, const N: usize> PartialOrd<&'b [U; N]> for Strided<T>
where
    T: FromPtr + 'a,
    T: PartialOrd<&'b U>,
    T::Borrow<'a>: PartialOrd<&'b U>,
{
    fn partial_cmp(&self, other: &&'b [U; N]) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Strided::from_slice(other.as_slice()))
    }
}
impl<'a, T, U, const N: usize> PartialOrd<[U; N]> for Strided<T>
where
    T: FromPtr + 'a,
    T: for<'b> PartialOrd<&'b U>,
    T::Borrow<'a>: for<'b> PartialOrd<&'b U>,
{
    fn partial_cmp(&self, other: &[U; N]) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other)
    }
}
impl<'a, 'b, T, U> PartialOrd<Strided<U>> for &'b [T]
where
    U: FromPtr + 'a,
    &'b T: PartialOrd<U> + PartialOrd<U::Borrow<'a>>,
{
    fn partial_cmp(&self, other: &Strided<U>) -> Option<std::cmp::Ordering> {
        Strided::from_slice(*self).partial_cmp(other)
    }
}
impl<'a, T, U> PartialOrd<Strided<U>> for [T]
where
    U: FromPtr + 'a,
    for<'b> &'b T: PartialOrd<U> + PartialOrd<U::Borrow<'a>>,
{
    fn partial_cmp(&self, other: &Strided<U>) -> Option<std::cmp::Ordering> {
        Strided::from_slice(self).partial_cmp(other)
    }
}
impl<'a, 'b, T, U, const N: usize> PartialOrd<Strided<U>> for &'b [T; N]
where
    U: FromPtr + 'a,
    &'b T: PartialOrd<U> + PartialOrd<U::Borrow<'a>>,
{
    fn partial_cmp(&self, other: &Strided<U>) -> Option<std::cmp::Ordering> {
        Strided::from_slice(self.as_slice()).partial_cmp(other)
    }
}
impl<'a, T, U, const N: usize> PartialOrd<Strided<U>> for [T; N]
where
    U: FromPtr + 'a,
    for<'b> &'b T: PartialOrd<U> + PartialOrd<U::Borrow<'a>>,
{
    fn partial_cmp(&self, other: &Strided<U>) -> Option<std::cmp::Ordering> {
        Strided::from_slice(self).partial_cmp(other)
    }
}

impl<'a, T> Ord for Strided<T>
where
    T: FromPtr + Ord + 'a,
    T::Borrow<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY:
        // It is most likely unsound considering interior mutability.
        // The problem is the cast to a broader lifetime that one can get in the implementation of the Ord trait,
        // And which is uncorrelated from a compiler point of view to the lifetime of &self or &other.
        // The bound T: Ord might be enough to make it sound because it is not possible to talk about
        //   this broader lifetime in the Ord implementation.
        // Even if this makes it sound, this would break when specialization becomes stable.
        let mut iter_a = unsafe { self.cast_as::<T::Borrow<'a>>().into_iter() };
        let mut iter_b = unsafe { other.cast_as::<T::Borrow<'a>>().into_iter() };

        loop {
            match (iter_a.next(), iter_b.next()) {
                (None, None) => return std::cmp::Ordering::Equal,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (Some(_), None) => return std::cmp::Ordering::Less,
                (Some(a), Some(b)) => match a.cmp(&b) {
                    std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
                },
            }
        }
    }
}

impl<'a, T> Hash for Strided<T>
where
    T: FromPtr + Hash + 'a,
    T::Borrow<'a>: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // SAFETY:
        // It is most likely unsound considering interior mutability.
        // The problem is the cast to a broader lifetime that one can get in the implementation of the Hash trait,
        // And which is uncorrelated from a compiler point of view to the lifetime of &self.
        // The bound T: Hash might be enough to make it sound because it is not possible to talk about
        //   this broader lifetime in the Hash implementation.
        // Even if this makes it sound, this would break when specialization becomes stable.
        for x in unsafe { self.cast_as::<T::Borrow<'a>>() } {
            x.hash(state);
        }
    }
}

impl<T: FromPtr> IntoIterator for Strided<T> {
    type Item = T;

    type IntoIter = Iter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

impl<'a, T: FromPtr> IntoIterator for &'a Strided<T> {
    type Item = T::Borrow<'a>;
    type IntoIter = Iter<T::Borrow<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.borrow())
    }
}

impl<'a, T: FromPtrMut> IntoIterator for &'a mut Strided<T> {
    type Item = T::BorrowMut<'a>;
    type IntoIter = Iter<T::BorrowMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.borrow_mut())
    }
}

pub struct Iter<T: FromPtr>(Strided<T>);

impl<T: FromPtr> Iterator for Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match std::mem::take(&mut self.0).into_split_first() {
            Some((first, slice)) => {
                self.0 = slice;
                Some(first)
            }
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.len()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match std::mem::take(&mut self.0).checked_into_split_at(n) {
            Some((_, tail)) => {
                self.0 = tail;
                self.next()
            }
            None => None,
        }
    }
}

impl<T: FromPtr> ExactSizeIterator for Iter<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: FromPtr> FusedIterator for Iter<T> {}
impl<T: FromPtr> DoubleEndedIterator for Iter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match std::mem::take(&mut self.0).into_split_last() {
            Some((last, slice)) => {
                self.0 = slice;
                Some(last)
            }
            None => None,
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.state.len = self.0.state.len.saturating_sub(n);
        self.next_back()
    }
}

impl<T> std::fmt::Debug for Strided<T>
where
    T: FromPtr + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(unsafe { self.unchecked_clone().into_iter() })
            .finish()
    }
}

impl<T> std::ops::Index<usize> for Strided<&'_ T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> std::ops::Index<usize> for Strided<&'_ mut T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> std::ops::IndexMut<usize> for Strided<&'_ mut T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T> std::ops::Index<usize> for Strided<&'_ [T]> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> std::ops::Index<usize> for Strided<&'_ mut [T]> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> std::ops::IndexMut<usize> for Strided<&'_ mut [T]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZero;

    use super::Strided;

    #[test]
    fn iter() {
        let mut array = [7, 5, 2, 8];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        assert!(slice.iter().copied().eq([7, 5, 2, 8]));
        assert!(slice.borrow().into_iter().copied().eq([7, 5, 2, 8]));

        assert!(slice.iter_mut().map(|x| *x).eq([7, 5, 2, 8]));
        assert!(slice.borrow_mut().into_iter().map(|x| *x).eq([7, 5, 2, 8]));

        assert!(slice
            .as_strided_ptr()
            .iter()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([7, 5, 2, 8]));
        assert!(slice
            .as_strided_ptr()
            .iter_mut()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([7, 5, 2, 8]));
        assert!(slice
            .as_strided_ptr()
            .into_iter()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([7, 5, 2, 8]));

        // MIRI: Check that there is no UB when materializing all mut references
        _ = slice.iter_mut().collect::<Vec<&mut _>>();
    }

    #[test]
    fn iter_rev() {
        let mut array = [7, 5, 2, 8];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        eprintln!("{:?}", slice.iter().rev().copied().collect::<Vec<_>>());

        assert!(slice.iter().rev().copied().eq([8, 2, 5, 7]));
        assert!(slice.borrow().into_iter().rev().copied().eq([8, 2, 5, 7]));

        assert!(slice.iter_mut().rev().map(|x| *x).eq([8, 2, 5, 7]));
        assert!(slice
            .borrow_mut()
            .into_iter()
            .rev()
            .map(|x| *x)
            .eq([8, 2, 5, 7]));

        assert!(slice
            .as_strided_ptr()
            .iter()
            .rev()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([8, 2, 5, 7]));
        assert!(slice
            .as_strided_ptr()
            .iter_mut()
            .rev()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([8, 2, 5, 7]));
        assert!(slice
            .as_strided_ptr()
            .into_iter()
            .rev()
            .map(|mut x| *unsafe { x.as_mut() })
            .eq([8, 2, 5, 7]));

        // MIRI: Check that there is no UB when materializing all mut references
        _ = slice.iter_mut().rev().collect::<Vec<&mut _>>();
    }

    #[test]
    fn eq() {
        let array = [1, 3, 6, 8];
        let mut array_mut = [1, 3, 6, 8];
        let slice = Strided::from_slice(array.as_slice());
        let mut slice_mut = Strided::from_slice(array_mut.as_mut_slice());

        // Checking that slice1 and slice2 points to different arrays ensure the comparison is done element by element
        assert_ne!(slice.as_non_null_ptr(), slice_mut.as_non_null_ptr());

        // Slice 1 EQ
        assert!(slice.iter().eq(slice.iter()));
        assert!(slice.iter().eq(slice.into_iter()));
        assert!(slice.iter().eq(slice_mut.iter()));
        assert!(slice.iter().eq(slice_mut.iter_mut()));
        assert!(slice.iter().eq(slice_mut.borrow_mut().into_iter()));

        assert_eq!(slice, slice);
        assert_eq!(slice, slice_mut);
        assert_eq!(slice, slice_mut.borrow());
        assert_eq!(slice, slice_mut.borrow_mut());
        assert_eq!(slice, Strided::from_slice([1u8, 3, 6, 8].as_slice()));
        assert_eq!(slice, Strided::from_slice([1u8, 3, 6, 8].as_mut_slice()));
        assert_eq!(slice, [1, 3, 6, 8]);
        assert_eq!(slice, &[1, 3, 6, 8]);
        assert_eq!(slice, *[1, 3, 6, 8].as_slice());
        assert_eq!(slice, *[1, 3, 6, 8].as_mut_slice());
        assert_eq!(slice, *[1u8, 3, 6, 8].as_slice());
        assert_eq!(slice, *[1u8, 3, 6, 8].as_mut_slice());
        assert!(slice.iter().eq(slice_mut.iter_mut()));

        // Slice 2 EQ
        assert!(slice_mut.iter().eq(slice.iter()));
        assert!(slice_mut.iter().eq(slice.into_iter()));
        assert!(slice_mut.iter().eq(slice_mut.iter()));

        assert_eq!(slice_mut, slice);
        assert_eq!(slice_mut, slice_mut);
        assert_eq!(slice_mut, slice_mut.borrow());
        assert_eq!(slice_mut, Strided::from_slice([1u8, 3, 6, 8].as_slice()));
        assert_eq!(
            slice_mut,
            Strided::from_slice([1u8, 3, 6, 8].as_mut_slice())
        );
        assert_eq!(slice_mut, [1, 3, 6, 8]);
        assert_eq!(slice_mut, &[1, 3, 6, 8]);
        assert_eq!(slice_mut, *[1, 3, 6, 8].as_slice());
        assert_eq!(slice_mut, *[1, 3, 6, 8].as_mut_slice());
        assert_eq!(slice_mut, *[1u8, 3, 6, 8].as_slice());
        assert_eq!(slice_mut, *[1u8, 3, 6, 8].as_mut_slice());
        assert_eq!(slice_mut, Strided::from_slice([1, 3, 6, 8].as_mut_slice()));

        // Slice 1 NE
        assert!(slice.iter().copied().ne([1, 3, 6, 9]));
        assert!(slice.into_iter().copied().ne([1, 3, 6, 9]));

        assert_ne!(slice, Strided::from_slice([].as_slice()));
        assert_ne!(slice, Strided::from_slice([1, 3, 6, 9].as_slice()));
        assert_ne!(slice, Strided::from_slice([1, 3, 6, 8, 9].as_slice()));
        assert_ne!(slice, Strided::from_slice([].as_mut_slice()));
        assert_ne!(slice, Strided::from_slice([1, 3, 6, 9].as_mut_slice()));
        assert_ne!(slice, Strided::from_slice([1, 3, 6, 8, 9].as_mut_slice()));

        assert_ne!(slice, []);
        assert_ne!(slice, [1, 3, 6, 9]);
        assert_ne!(slice, [1, 3, 6, 8, 9]);
        assert_ne!(slice, &[]);
        assert_ne!(slice, &[1, 3, 6, 9]);
        assert_ne!(slice, &[1, 3, 6, 8, 9]);
        assert_ne!(slice, *[].as_slice());
        assert_ne!(slice, *[1, 3, 6, 9].as_slice());
        assert_ne!(slice, *[1, 3, 6, 8, 9].as_slice());
        assert_ne!(slice, *[].as_mut_slice());
        assert_ne!(slice, *[1, 3, 6, 9].as_mut_slice());
        assert_ne!(slice, *[1, 3, 6, 8, 9].as_mut_slice());

        // Slice 2 NE
        assert!(slice_mut.iter().copied().ne([1, 3, 6, 9]));
        assert!(slice_mut.iter_mut().map(|x| *x).ne([1, 3, 6, 9]));
        assert!(slice_mut
            .borrow_mut()
            .into_iter()
            .map(|x| *x)
            .ne([1, 3, 6, 9]));

        assert_ne!(slice_mut, Strided::from_slice([].as_slice()));
        assert_ne!(slice_mut, Strided::from_slice([1, 3, 6, 9].as_slice()));
        assert_ne!(slice_mut, Strided::from_slice([1, 3, 6, 8, 9].as_slice()));
        assert_ne!(slice_mut, Strided::from_slice([].as_mut_slice()));
        assert_ne!(slice_mut, Strided::from_slice([1, 3, 6, 9].as_mut_slice()));
        assert_ne!(
            slice_mut,
            Strided::from_slice([1, 3, 6, 8, 9].as_mut_slice())
        );

        assert_ne!(slice_mut, []);
        assert_ne!(slice_mut, [1, 3, 6, 9]);
        assert_ne!(slice_mut, [1, 3, 6, 8, 9]);
        assert_ne!(slice_mut, &[]);
        assert_ne!(slice_mut, &[1, 3, 6, 9]);
        assert_ne!(slice_mut, &[1, 3, 6, 8, 9]);
        assert_ne!(slice_mut, *[].as_slice());
        assert_ne!(slice_mut, *[1, 3, 6, 9].as_slice());
        assert_ne!(slice_mut, *[1, 3, 6, 8, 9].as_slice());
        assert_ne!(slice_mut, *[].as_mut_slice());
        assert_ne!(slice_mut, *[1, 3, 6, 9].as_mut_slice());
        assert_ne!(slice_mut, *[1, 3, 6, 8, 9].as_mut_slice());
    }

    #[test]
    fn access() {
        let array = [1, 3, 6, 8];
        let mut array_mut = [1, 3, 6, 8];
        let slice = Strided::from_slice(array.as_slice());
        let mut slice_mut = Strided::from_slice(array_mut.as_mut_slice());

        // Ref access slice
        assert_eq!(*unsafe { slice.unchecked_get(0) }, 1);
        assert_eq!(*unsafe { slice.unchecked_get(1) }, 3);
        assert_eq!(*unsafe { slice.unchecked_get(2) }, 6);
        assert_eq!(*unsafe { slice.unchecked_get(3) }, 8);
        assert_eq!(slice.checked_get(0).copied(), Some(1));
        assert_eq!(slice.checked_get(1).copied(), Some(3));
        assert_eq!(slice.checked_get(2).copied(), Some(6));
        assert_eq!(slice.checked_get(3).copied(), Some(8));
        assert_eq!(slice.checked_get(4).copied(), None);
        assert_eq!(*slice.get(0), 1);
        assert_eq!(*slice.get(1), 3);
        assert_eq!(*slice.get(2), 6);
        assert_eq!(*slice.get(3), 8);

        // Ref access slice_mut
        assert_eq!(*unsafe { slice_mut.unchecked_get(0) }, 1);
        assert_eq!(*unsafe { slice_mut.unchecked_get(1) }, 3);
        assert_eq!(*unsafe { slice_mut.unchecked_get(2) }, 6);
        assert_eq!(*unsafe { slice_mut.unchecked_get(3) }, 8);
        assert_eq!(slice_mut.checked_get(0).copied(), Some(1));
        assert_eq!(slice_mut.checked_get(1).copied(), Some(3));
        assert_eq!(slice_mut.checked_get(2).copied(), Some(6));
        assert_eq!(slice_mut.checked_get(3).copied(), Some(8));
        assert_eq!(slice_mut.checked_get(4).copied(), None);
        assert_eq!(*slice_mut.get(0), 1);
        assert_eq!(*slice_mut.get(1), 3);
        assert_eq!(*slice_mut.get(2), 6);
        assert_eq!(*slice_mut.get(3), 8);

        // Mut access slice_mut
        assert_eq!(*unsafe { slice_mut.unchecked_get_mut(0) }, 1);
        assert_eq!(*unsafe { slice_mut.unchecked_get_mut(1) }, 3);
        assert_eq!(*unsafe { slice_mut.unchecked_get_mut(2) }, 6);
        assert_eq!(*unsafe { slice_mut.unchecked_get_mut(3) }, 8);
        assert_eq!(slice_mut.checked_get_mut(0).copied(), Some(1));
        assert_eq!(slice_mut.checked_get_mut(1).copied(), Some(3));
        assert_eq!(slice_mut.checked_get_mut(2).copied(), Some(6));
        assert_eq!(slice_mut.checked_get_mut(3).copied(), Some(8));
        assert_eq!(slice_mut.checked_get_mut(4).copied(), None);
        assert_eq!(*slice_mut.get_mut(0), 1);
        assert_eq!(*slice_mut.get_mut(1), 3);
        assert_eq!(*slice_mut.get_mut(2), 6);
        assert_eq!(*slice_mut.get_mut(3), 8);

        *unsafe { slice_mut.unchecked_get_mut(1) } = 12;
        *slice_mut.checked_get_mut(2).unwrap() = 14;
        *slice_mut.get_mut(3) = 17;

        assert_eq!(slice_mut, [1, 12, 14, 17]);
        assert_eq!(slice_mut.borrow(), [1, 12, 14, 17]);
        assert_eq!(slice_mut.borrow_mut(), [1, 12, 14, 17]);

        *unsafe { slice_mut.borrow_mut().unchecked_get_mut(0) } = 23;
        *slice_mut.borrow_mut().checked_get_mut(1).unwrap() = 26;
        *slice_mut.borrow_mut().get_mut(2) = 27;

        assert_eq!(slice_mut, [23, 26, 27, 17]);
        assert_eq!(slice_mut.borrow(), [23, 26, 27, 17]);
        assert_eq!(slice_mut.borrow_mut(), [23, 26, 27, 17]);
    }

    #[test]
    #[should_panic]
    fn access_oob() {
        let slice = Strided::from_slice([0, 1].as_slice());

        slice.get(2);
    }

    #[test]
    fn partial() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        for (start, len, step_by, reference) in [
            (0, 0, 1, [].as_slice()),
            (0, 2, 3, &[0, 3]),
            (1, 3, 2, &[1, 3, 5]),
            (1, 3, -2, &[9, 7, 5]),
            (1, 5, 2, &[1, 3, 5, 7, 9]),
            (1, 5, -2, &[9, 7, 5, 3, 1]),
        ] {
            assert_eq!(
                unsafe {
                    slice
                        .borrow_mut()
                        .unchecked_into_partial(start, len, step_by)
                },
                reference
            );
            assert_eq!(
                unsafe { slice.borrow().unchecked_into_partial(start, len, step_by) },
                reference
            );
            assert_eq!(
                unsafe { slice.unchecked_partial_mut(start, len, step_by) },
                reference
            );
            assert_eq!(
                unsafe { slice.unchecked_partial(start, len, step_by) },
                reference
            );
            assert_eq!(
                unsafe { slice.borrow().unchecked_partial(start, len, step_by) },
                reference
            );
            assert_eq!(
                slice
                    .borrow_mut()
                    .into_partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice
                    .borrow()
                    .into_partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice.partial_mut(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice.partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice
                    .borrow()
                    .partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
        }
        for (start, len, step_by, reference) in [
            (10, 1, 1, [].as_slice()),
            (1, 10, 2, &[1, 3, 5, 7, 9]),
            (1, 10, -2, &[9, 7, 5, 3, 1]),
            (100, 10, -2, &[]),
            (usize::MAX, 10, 1, &[]),
            (1, 10, isize::MAX, &[1]),
            (1, 10, isize::MIN, &[9]),
        ] {
            assert_eq!(
                slice
                    .borrow_mut()
                    .into_partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice
                    .borrow()
                    .into_partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice.partial_mut(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice.partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
            assert_eq!(
                slice
                    .borrow()
                    .partial(start, len, NonZero::new(step_by).unwrap()),
                reference
            );
        }
    }

    #[test]
    fn split_at() {
        #[allow(static_mut_refs)]
        unsafe fn reference_mut(
            i: usize,
        ) -> (Strided<&'static mut i32>, Strided<&'static mut i32>) {
            static mut ARRAY: [i32; 4] = [0, 1, 2, 3];
            let slice = unsafe { ARRAY.as_mut_slice() };
            let (a, b) = slice.split_at_mut(i);

            (Strided::from_slice(a), Strided::from_slice(b))
        }

        fn reference(i: usize) -> (Strided<&'static i32>, Strided<&'static i32>) {
            let (a, b) = unsafe { reference_mut(i) };

            (a.into_borrow(), b.into_borrow())
        }

        let mut array = [0, 1, 2, 3];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        for i in 0..=4 {
            assert_eq!(
                unsafe { slice.borrow().unchecked_into_split_at(i) },
                reference(i)
            );
            assert_eq!(slice.borrow().into_split_at(i), reference(i));
            assert_eq!(slice.borrow().checked_into_split_at(i), Some(reference(i)));

            assert_eq!(
                unsafe { slice.borrow().unchecked_split_at(i) },
                reference(i)
            );
            assert_eq!(slice.borrow().split_at(i), reference(i));
            assert_eq!(slice.borrow().checked_split_at(i), Some(reference(i)));

            assert_eq!(
                unsafe { slice.borrow_mut().unchecked_into_split_at(i) },
                unsafe { reference_mut(i) }
            );
            assert_eq!(slice.borrow_mut().into_split_at(i), unsafe {
                reference_mut(i)
            });
            assert_eq!(
                slice.borrow_mut().checked_into_split_at(i),
                Some(unsafe { reference_mut(i) })
            );

            assert_eq!(unsafe { slice.unchecked_split_at_mut(i) }, unsafe {
                reference_mut(i)
            });
            assert_eq!(slice.split_at_mut(i), unsafe { reference_mut(i) });
            assert_eq!(
                slice.checked_split_at_mut(i),
                Some(unsafe { reference_mut(i) })
            );

            assert_eq!(unsafe { slice.unchecked_split_at(i) }, reference(i));
            assert_eq!(slice.split_at(i), reference(i));
            assert_eq!(slice.checked_split_at(i), Some(reference(i)));

            // MIRI: Check that there is no UB when materializing all mut references
            let (a, b) = slice.split_at_mut(i);
            let a = a.into_iter().collect::<Vec<&mut _>>();
            let b = b.into_iter().collect::<Vec<&mut _>>();

            std::mem::drop(a);
            std::mem::drop(b);
        }

        assert_eq!(slice.borrow().checked_into_split_at(5), None);
        assert_eq!(slice.borrow().checked_split_at(5), None);

        assert_eq!(slice.borrow_mut().checked_into_split_at(5), None);
        assert_eq!(slice.checked_split_at(5), None);
        assert_eq!(slice.checked_split_at_mut(5), None);
    }

    #[test]
    #[should_panic]
    fn split_at_oob() {
        let slice = Strided::from_slice(&[1, 2]);
        slice.split_at(3);
    }

    #[test]
    fn chunk() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        let (chunks, remainder) = slice.borrow_mut().into_chunks(NonZero::new(3).unwrap());
        assert_eq!(chunks, [[0, 1, 2], [3, 4, 5], [6, 7, 8]]);
        assert_eq!(remainder, [9]);

        let (chunks, remainder) = slice.as_chunks_mut(NonZero::new(3).unwrap());
        assert_eq!(chunks, [[0, 1, 2], [3, 4, 5], [6, 7, 8]]);
        assert_eq!(remainder, [9]);

        let (chunks, remainder) = slice.as_chunks(NonZero::new(3).unwrap());
        assert_eq!(chunks, [[0, 1, 2], [3, 4, 5], [6, 7, 8]]);
        assert_eq!(remainder, [9]);

        let (chunks, remainder) = slice.borrow_mut().into_chunks(NonZero::new(2).unwrap());
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.as_chunks_mut(NonZero::new(2).unwrap());
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.as_chunks(NonZero::new(2).unwrap());
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.borrow_mut().into_chunks(NonZero::new(10).unwrap());
        assert_eq!(chunks, [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.as_chunks_mut(NonZero::new(10).unwrap());
        assert_eq!(chunks, [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.as_chunks(NonZero::new(10).unwrap());
        assert_eq!(chunks, [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]]);
        assert_eq!(remainder, []);

        let (chunks, remainder) = slice.borrow_mut().into_chunks(NonZero::new(11).unwrap());
        assert_eq!(chunks, [[0i32; 11]; 0]);
        assert_eq!(remainder, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let (chunks, remainder) = slice.as_chunks_mut(NonZero::new(11).unwrap());
        assert_eq!(chunks, [[0i32; 11]; 0]);
        assert_eq!(remainder, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let (chunks, remainder) = slice.as_chunks(NonZero::new(11).unwrap());
        assert_eq!(chunks, [[0i32; 11]; 0]);
        assert_eq!(remainder, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let chunks = unsafe {
            slice
                .borrow_mut()
                .unchecked_into_chunks(NonZero::new(2).unwrap())
        };
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);

        let chunks = unsafe { slice.unchecked_as_chunks_mut(NonZero::new(2).unwrap()) };
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);
        let chunks = unsafe { slice.unchecked_as_chunks(NonZero::new(2).unwrap()) };
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);

        let (chunks, remainder) =
            unsafe { slice.unchecked_partial(0, 0, 1) }.into_chunks(NonZero::new(2).unwrap());
        assert_eq!(chunks, [[0i32; 2]; 0]);
        assert_eq!(remainder, [0i32; 0]);

        // MIRI: Check that there is no UB when materializing all mut references
        _ = unsafe { slice.unchecked_as_chunks_mut(NonZero::new(2).unwrap()) }
            .into_iter()
            .map(|chunk| chunk.into_iter().collect::<Vec<&mut _>>())
            .collect::<Vec<Vec<&mut _>>>();
    }

    #[test]
    fn rchunk() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        let (remainder, chunks) = slice.borrow_mut().into_rchunks(NonZero::new(3).unwrap());
        assert_eq!(remainder, [0]);
        assert_eq!(chunks, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        let (remainder, chunks) = slice.as_rchunks_mut(NonZero::new(3).unwrap());
        assert_eq!(remainder, [0]);
        assert_eq!(chunks, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        let (remainder, chunks) = slice.as_rchunks(NonZero::new(3).unwrap());
        assert_eq!(remainder, [0]);
        assert_eq!(chunks, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        let (remainder, chunks) = slice.borrow_mut().into_rchunks(NonZero::new(2).unwrap());
        assert_eq!(remainder, []);
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);

        let (remainder, chunks) = slice.as_rchunks_mut(NonZero::new(2).unwrap());
        assert_eq!(remainder, []);
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);

        let (remainder, chunks) = slice.as_rchunks(NonZero::new(2).unwrap());
        assert_eq!(remainder, []);
        assert_eq!(chunks, [[0, 1], [2, 3], [4, 5], [6, 7], [8, 9]]);
    }

    #[test]
    fn deinterleave() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut slice = Strided::from_slice(array.as_mut_slice());

        assert_eq!(
            slice.borrow_mut().into_deinterleave_array::<2>(),
            [[0, 2, 4, 6, 8], [1, 3, 5, 7, 9]]
        );
        assert_eq!(
            slice.deinterleave_array_mut::<2>(),
            [[0, 2, 4, 6, 8], [1, 3, 5, 7, 9]]
        );
        assert_eq!(
            slice.deinterleave_array::<2>(),
            [[0, 2, 4, 6, 8], [1, 3, 5, 7, 9]]
        );

        assert_eq!(
            slice.borrow_mut().into_deinterleave_array::<4>(),
            [[0, 4, 8].as_slice(), &[1, 5, 9], &[2, 6], &[3, 7]]
        );
        assert_eq!(
            slice.deinterleave_array_mut::<4>(),
            [[0, 4, 8].as_slice(), &[1, 5, 9], &[2, 6], &[3, 7]]
        );
        assert_eq!(
            slice.deinterleave_array::<4>(),
            [[0, 4, 8].as_slice(), &[1, 5, 9], &[2, 6], &[3, 7]]
        );
    }

    #[test]
    fn transpose() {
        let mut array = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11]];

        println!(
            "{:?}",
            Strided::<Strided<std::ptr::NonNull<i32>>>::from(
                Strided::<std::ptr::NonNull<[i32; 3]>>::from(array.as_slice())
            )
            .state
        );

        let mut matrix = Strided::from_matrix(&mut array);

        for row in &matrix {
            for cell in row {
                print!(" {cell}");
            }
            println!()
        }

        assert_eq!(matrix, [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11]]);

        assert_eq!(
            matrix.borrow_mut().into_transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
        assert_eq!(
            *matrix.borrow_mut().transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
        assert_eq!(
            matrix.as_transpose01_mut(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
        assert_eq!(
            matrix.as_transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );

        assert_eq!(
            matrix.borrow().into_transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
        assert_eq!(
            *matrix.borrow().transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
        assert_eq!(
            matrix.borrow().as_transpose01(),
            [[0, 3, 6, 9], [1, 4, 7, 10], [2, 5, 8, 11]]
        );
    }

    #[test]
    fn blocks() {
        let mut array = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [10, 11, 12, 13, 14, 15, 16, 17],
            [20, 21, 22, 23, 24, 25, 26, 27],
            [30, 31, 32, 33, 34, 35, 36, 37],
            [40, 41, 42, 43, 44, 45, 46, 47],
            [50, 51, 52, 53, 54, 55, 56, 57],
            [60, 61, 62, 63, 64, 65, 66, 67],
            [70, 71, 72, 73, 74, 75, 76, 77],
        ];
        let mut matrix = Strided::from_matrix(&mut array);

        let reference2_blocks = [
            [
                [[0, 1], [10, 11]],
                [[2, 3], [12, 13]],
                [[4, 5], [14, 15]],
                [[6, 7], [16, 17]],
            ],
            [
                [[20, 21], [30, 31]],
                [[22, 23], [32, 33]],
                [[24, 25], [34, 35]],
                [[26, 27], [36, 37]],
            ],
            [
                [[40, 41], [50, 51]],
                [[42, 43], [52, 53]],
                [[44, 45], [54, 55]],
                [[46, 47], [56, 57]],
            ],
            [
                [[60, 61], [70, 71]],
                [[62, 63], [72, 73]],
                [[64, 65], [74, 75]],
                [[66, 67], [76, 77]],
            ],
        ];
        let reference2_remaining0 = [[[0; 0]; 0]; 4];
        let reference2_remaining1 = [[[0; 0]; 2]; 4];
        let reference2_remaining01 = [[0; 0]; 0];

        let blocked = matrix
            .borrow_mut()
            .into_blocks(NonZero::new(2).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference2_blocks);
        assert_eq!(blocked.remaining0, reference2_remaining0);
        assert_eq!(blocked.remaining1, reference2_remaining1);
        assert_eq!(blocked.remaining01, reference2_remaining01);

        let blocked = matrix.as_blocks_mut(NonZero::new(2).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference2_blocks);
        assert_eq!(blocked.remaining0, reference2_remaining0);
        assert_eq!(blocked.remaining1, reference2_remaining1);
        assert_eq!(blocked.remaining01, reference2_remaining01);

        let blocked = matrix.as_blocks(NonZero::new(2).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference2_blocks);
        assert_eq!(blocked.remaining0, reference2_remaining0);
        assert_eq!(blocked.remaining1, reference2_remaining1);
        assert_eq!(blocked.remaining01, reference2_remaining01);

        let blocks = unsafe {
            matrix
                .borrow_mut()
                .unchecked_into_blocks(NonZero::new(2).unwrap(), NonZero::new(2).unwrap())
        };
        assert_eq!(blocks, reference2_blocks);

        let blocks = unsafe {
            matrix.unchecked_as_blocks_mut(NonZero::new(2).unwrap(), NonZero::new(2).unwrap())
        };
        assert_eq!(blocks, reference2_blocks);

        let blocks = unsafe {
            matrix.unchecked_as_blocks(NonZero::new(2).unwrap(), NonZero::new(2).unwrap())
        };
        assert_eq!(blocks, reference2_blocks);

        let reference23_blocks = [
            [[[0, 1, 2], [10, 11, 12]], [[3, 4, 5], [13, 14, 15]]],
            [[[20, 21, 22], [30, 31, 32]], [[23, 24, 25], [33, 34, 35]]],
            [[[40, 41, 42], [50, 51, 52]], [[43, 44, 45], [53, 54, 55]]],
            [[[60, 61, 62], [70, 71, 72]], [[63, 64, 65], [73, 74, 75]]],
        ];
        let reference23_remaining0 = [[[0; 0]; 0]; 2];
        let reference23_remaining1 = [
            [[6, 7], [16, 17]],
            [[26, 27], [36, 37]],
            [[46, 47], [56, 57]],
            [[66, 67], [76, 77]],
        ];
        let reference23_remaining01 = [[0; 0]; 0];

        let blocked = matrix
            .borrow_mut()
            .into_blocks(NonZero::new(2).unwrap(), NonZero::new(3).unwrap());
        assert_eq!(blocked.blocks, reference23_blocks);
        assert_eq!(blocked.remaining0, reference23_remaining0);
        assert_eq!(blocked.remaining1, reference23_remaining1);
        assert_eq!(blocked.remaining01, reference23_remaining01);

        let blocked = matrix.as_blocks_mut(NonZero::new(2).unwrap(), NonZero::new(3).unwrap());
        assert_eq!(blocked.blocks, reference23_blocks);
        assert_eq!(blocked.remaining0, reference23_remaining0);
        assert_eq!(blocked.remaining1, reference23_remaining1);
        assert_eq!(blocked.remaining01, reference23_remaining01);

        let blocked = matrix.as_blocks(NonZero::new(2).unwrap(), NonZero::new(3).unwrap());
        assert_eq!(blocked.blocks, reference23_blocks);
        assert_eq!(blocked.remaining0, reference23_remaining0);
        assert_eq!(blocked.remaining1, reference23_remaining1);
        assert_eq!(blocked.remaining01, reference23_remaining01);

        let reference32_blocks = [
            [
                [[0, 1], [10, 11], [20, 21]],
                [[2, 3], [12, 13], [22, 23]],
                [[4, 5], [14, 15], [24, 25]],
                [[6, 7], [16, 17], [26, 27]],
            ],
            [
                [[30, 31], [40, 41], [50, 51]],
                [[32, 33], [42, 43], [52, 53]],
                [[34, 35], [44, 45], [54, 55]],
                [[36, 37], [46, 47], [56, 57]],
            ],
        ];
        let reference32_remaining0 = [
            [[60, 61], [70, 71]],
            [[62, 63], [72, 73]],
            [[64, 65], [74, 75]],
            [[66, 67], [76, 77]],
        ];
        let reference32_remaining1 = [[[0; 0]; 3]; 2];
        let reference32_remaining01 = [[0; 0]; 2];

        let blocked = matrix
            .borrow_mut()
            .into_blocks(NonZero::new(3).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference32_blocks);
        assert_eq!(blocked.remaining0, reference32_remaining0);
        assert_eq!(blocked.remaining1, reference32_remaining1);
        assert_eq!(blocked.remaining01, reference32_remaining01);

        let blocked = matrix.as_blocks_mut(NonZero::new(3).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference32_blocks);
        assert_eq!(blocked.remaining0, reference32_remaining0);
        assert_eq!(blocked.remaining1, reference32_remaining1);
        assert_eq!(blocked.remaining01, reference32_remaining01);

        let blocked = matrix.as_blocks(NonZero::new(3).unwrap(), NonZero::new(2).unwrap());
        assert_eq!(blocked.blocks, reference32_blocks);
        assert_eq!(blocked.remaining0, reference32_remaining0);
        assert_eq!(blocked.remaining1, reference32_remaining1);
        assert_eq!(blocked.remaining01, reference32_remaining01);
    }
}
