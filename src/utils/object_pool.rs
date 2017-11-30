use std::borrow::Borrow;
use super::{Handle, HandlePool, HandleIter};

/// A named object collections. Every time u create or free a handle, a
/// attached instance `T` will be created/ freed.
pub struct ObjectPool<T: Sized> {
    handles: HandlePool,
    values: Vec<Option<T>>,
}

impl<T: Sized> ObjectPool<T> {
    /// Constructs a new, empty `ObjectPool`.
    pub fn new() -> Self {
        ObjectPool {
            handles: HandlePool::new(),
            values: Vec::new(),
        }
    }

    /// Constructs a new `ObjectPool` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ObjectPool {
            handles: HandlePool::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Creates a `T` and named it with `Handle`.
    pub fn create(&mut self, value: T) -> Handle {
        let handle = self.handles.create();

        if handle.index() >= self.values.len() as u32 {
            self.values.push(Some(value));
        } else {
            self.values[handle.index() as usize] = Some(value);
        }

        handle
    }

    /// Returns mutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        if self.handles.is_alive(handle) {
            self.values[handle.index() as usize].as_mut()
        } else {
            None
        }
    }

    /// Returns immutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get<H>(&self, handle: H) -> Option<&T>
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        if self.handles.is_alive(handle) {
            self.values[handle.index() as usize].as_ref()
        } else {
            None
        }
    }

    #[inline]
    pub fn is_alive<H>(&self, handle: H) -> bool
        where H: Borrow<Handle>
    {
        self.handles.is_alive(handle)
    }

    /// Recycles the value with name `Handle`.
    #[inline]
    pub fn free<H>(&mut self, handle: H) -> Option<T>
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        if self.handles.free(handle) {
            let mut v = None;
            ::std::mem::swap(&mut v, &mut self.values[handle.index() as usize]);
            v
        } else {
            None
        }
    }

    /// Returns the total number of alive handle in this `ObjectPool`.
    #[inline]
    pub fn len(&self) -> usize {
        self.handles.size()
    }

    /// Returns an iterator over the `ObjectPool`.
    #[inline]
    pub fn iter(&self) -> HandleIter {
        self.handles.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut set = ObjectPool::<i32>::new();

        let e1 = set.create(3);
        assert_eq!(set.get(e1), Some(&3));
        assert_eq!(set.len(), 1);
        assert_eq!(set.free(e1), Some(3));
        assert_eq!(set.len(), 0);
        assert_eq!(set.get(e1), None);
        assert_eq!(set.free(e1), None);
        assert_eq!(set.len(), 0);
    }
}