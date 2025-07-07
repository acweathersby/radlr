use core::ops::{Index, IndexMut};
use std::fmt::Debug;

/// A vector which derives its initial capacity from the stack. The data is
/// moved to the heap if the number of elements pushed to the vector exceed
/// the stack capacity.
///
/// Since the data is stored on the stack, care should be given to prevent
/// stack overflow. Generally, a limited number of items in that do not
/// exceed 1kb in total allocated space should be safe to store on the
/// stack.
pub struct StackVec<const STACK_SIZE: usize, T: Sized> {
  inner:       [T; STACK_SIZE],
  vec:         Option<Vec<T>>,
  allocations: usize,
}

impl<const STACK_SIZE: usize, T: Debug> Debug for StackVec<STACK_SIZE, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("StackVec");

    s.field("data", &self.as_slice());

    s.finish()
  }
}

impl<const STACK_SIZE: usize, T> Index<usize> for StackVec<STACK_SIZE, T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    if index >= self.len() {
      panic!("Index {index} is out of range of 0..{}", self.len());
    }

    if let Some(vec) = &self.vec {
      &vec[index]
    } else {
      &self.inner[index]
    }
  }
}

impl<const STACK_SIZE: usize, T> IndexMut<usize> for StackVec<STACK_SIZE, T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    if index >= self.len() {
      panic!("Index {index} is out of range of 0..{}", self.len());
    }

    if let Some(vec) = &mut self.vec {
      &mut vec[index]
    } else {
      &mut self.inner[index]
    }
  }
}

impl<const STACK_SIZE: usize, T: Ord> StackVec<STACK_SIZE, T> {
  pub fn sort(&mut self) {
    self.as_mut_slice().sort();
  }
}

impl<const STACK_SIZE: usize, T> AsRef<[T]> for StackVec<STACK_SIZE, T> {
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<const STACK_SIZE: usize, T> AsMut<[T]> for StackVec<STACK_SIZE, T> {
  fn as_mut(&mut self) -> &mut [T] {
    self.as_mut_slice()
  }
}

impl<const STACK_SIZE: usize, T: Sized + Eq + Debug> StackVec<STACK_SIZE, T> {
  pub fn push_unique(&mut self, item: T) -> bool {
    let len = self.len();
    let mut i = 0;
    let slice = self.as_slice();
    while i < len {
      if slice[i] == item {
        return false;
      }
      i += 1;
    }
    self.push(item);
    true
  }

  #[inline(never)]
  pub fn new() -> Self {
    let inner: [T; STACK_SIZE] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    let out = Self { inner, vec: None, allocations: 0 };

    out
  }
}

impl<const STACK_SIZE: usize, T: Sized> StackVec<STACK_SIZE, T> {
  /// Converts the stack vec into a regular vector, consuming the stack vec in
  /// the process.
  pub fn to_vec(self) -> Vec<T> {
    if let Some(vec) = self.vec {
      vec
    } else {
      let data = self.inner;
      let mut vec = Vec::<T>::with_capacity(self.allocations);
      unsafe {
        core::ptr::copy(data.as_ptr(), vec.as_mut_ptr(), self.allocations);
        vec.set_len(self.allocations);
      };
      core::mem::forget(data);
      vec
    }
  }

  pub fn len(&self) -> usize {
    if let Some(vec) = &self.vec {
      vec.len()
    } else {
      self.allocations
    }
  }

  pub fn iter<'stack>(&'stack self) -> StackVecIterator<'stack, STACK_SIZE, T> {
    StackVecIterator { inner: self, tracker: 0, len: self.len() }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn push(&mut self, mut element: T) {
    if let Some(vec) = &mut self.vec {
      vec.push(element)
    } else {
      if self.allocations < STACK_SIZE {
        core::mem::swap(&mut self.inner[self.allocations], &mut element);
        core::mem::forget(element);
        self.allocations += 1;
      } else {
        unsafe {
          let mut vec = Vec::<T>::with_capacity(self.allocations);
          core::ptr::copy(self.inner.as_ptr(), vec.as_mut_ptr(), self.allocations);
          vec.set_len(self.allocations);
          vec.push(element);
          self.vec = Some(vec);
        }
      }
    }
  }

  #[inline(never)]
  pub fn as_slice(&self) -> &[T] {
    if let Some(vec) = &self.vec {
      vec.as_slice()
    } else {
      unsafe { core::slice::from_raw_parts(self.inner.as_ptr(), self.allocations) }
    }
  }

  #[inline(never)]
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    if let Some(vec) = &mut self.vec {
      vec.as_mut_slice()
    } else {
      unsafe { core::slice::from_raw_parts_mut(self.inner.as_mut_ptr(), self.allocations) }
    }
  }
}

pub struct StackVecIterator<'stack, const STACK_SIZE: usize, T> {
  inner:   &'stack StackVec<STACK_SIZE, T>,
  tracker: usize,
  len:     usize,
}

impl<'stack, const STACK_SIZE: usize, T> Iterator for StackVecIterator<'stack, STACK_SIZE, T> {
  type Item = &'stack T;

  fn next(&mut self) -> Option<Self::Item> {
    if self.tracker < self.len {
      let index = self.tracker;
      self.tracker += 1;
      Some(self.inner.index(index))
    } else {
      None
    }
  }
}

pub struct StackVecIteratorMut<'stack, const STACK_SIZE: usize, T> {
  inner:   &'stack mut StackVec<STACK_SIZE, T>,
  tracker: usize,
  len:     usize,
}

impl<'stack, const STACK_SIZE: usize, T> Iterator for StackVecIteratorMut<'stack, STACK_SIZE, T> {
  type Item = &'stack mut T;

  fn next(&mut self) -> Option<Self::Item> {
    if self.tracker < self.len {
      let index = self.tracker;
      self.tracker += 1;
      let inner: *mut StackVec<STACK_SIZE, T> = self.inner as *mut _;
      if let Some(vec) = &mut (unsafe { &mut *inner }).vec {
        Some(&mut vec[index])
      } else {
        Some(&mut (unsafe { &mut *inner }).inner[index])
      }
    } else {
      None
    }
  }
}
