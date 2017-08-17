use std::ops::Deref;
use std::borrow::Borrow;

/// `HandleIndex` type is arbitrary. Keeping it 32-bits allows for
/// a single 64-bits word per `Handle`.
pub type HandleIndex = u32;

/// `Handle` is made up of two field, `index` and `version`. `index` are
/// usually used to indicated address into some kind of space. This value
/// is recycled when an `Handle` is freed to save address. However, this
/// means that you could end up with two different `Handle` with identical
/// indices. We solve this by introducing `version`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle {
    index: HandleIndex,
    version: HandleIndex,
}

impl Handle {
    /// Constructs a new `Handle`.
    #[inline]
    pub fn new(index: HandleIndex, version: HandleIndex) -> Self {
        Handle {
            index: index,
            version: version,
        }
    }

    /// Constructs a nil/uninitialized `Handle`.
    #[inline]
    pub fn nil() -> Self {
        Handle {
            index: 0,
            version: 0,
        }
    }

    /// Returns true if this `Handle` has been initialized.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.index > 0 || self.version > 0
    }

    /// Invalidate this `Handle` to default value.
    #[inline]
    pub fn invalidate(&mut self) {
        self.index = 0;
        self.version = 0;
    }

    /// Returns index value.
    #[inline]
    pub fn index(&self) -> HandleIndex {
        self.index
    }

    /// Returns version value.
    #[inline]
    pub fn version(&self) -> HandleIndex {
        self.version
    }
}

impl Deref for Handle {
    type Target = HandleIndex;

    fn deref(&self) -> &HandleIndex {
        &self.index
    }
}

impl Borrow<HandleIndex> for Handle {
    fn borrow(&self) -> &HandleIndex {
        &self.index
    }
}

impl<'a> Borrow<HandleIndex> for &'a Handle {
    fn borrow(&self) -> &HandleIndex {
        &self.index
    }
}

macro_rules! impl_handle {
    ($name: ident) => (
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

        pub struct $name (Handle);

        impl From<Handle> for $name {
            fn from(handle: Handle) -> Self {
                $name(handle)
            }
        }

        impl $crate::std::ops::Deref for $name {
            type Target = Handle;
            fn deref(&self) -> &Handle {
                &self.0
            }
        }

        impl $crate::std::borrow::Borrow<Handle> for $name {
            fn borrow(&self) -> &Handle {
                &self.0
            }
        }
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut h2 = Handle::new(2, 4);
        assert_eq!(h2.index, 2);
        assert_eq!(h2.version, 4);
        assert!(h2.is_valid());
        assert_eq!(*h2, 2);

        h2.invalidate();
        assert_eq!(h2.index, 0);
        assert_eq!(h2.version, 0);
        assert!(!h2.is_valid());
        assert_eq!(*h2, 0);
    }

    #[test]
    fn container() {
        use std::collections::HashSet;
        let h1 = Handle::new(1, 1);
        let h2 = Handle::new(1, 2);
        let h3 = Handle::new(2, 2);
        let h4 = Handle::new(1, 1);

        let mut map = HashSet::new();
        assert_eq!(map.insert(h1), true);
        assert_eq!(map.contains(&h1), true);
        assert_eq!(map.insert(h4), false);
        assert_eq!(map.contains(&h4), true);
        assert_eq!(map.insert(h2), true);
        assert_eq!(map.insert(h3), true);
    }

    impl_handle!(TypeSafeHandle);
    #[test]
    fn type_safe_handle() {
        let h1 = TypeSafeHandle::default();
        assert_eq!(h1, TypeSafeHandle::from(Handle::default()));

        let h2 = TypeSafeHandle(Handle::default());
        assert_eq!(*h2, Handle::default());
    }
}
