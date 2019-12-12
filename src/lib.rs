use std::{
    borrow::{Borrow, BorrowMut},
    mem,
    ops::{Deref, DerefMut},
};

/// Marker for private C structures.
///
/// It is a common practice in C not to hide contents of types by defining them
/// in implementation files (.c), and only expose a typedef. Such structures
/// should be represented with `Private`.
///
/// ## Example
///
/// Let's assume that the library we're writing bindings to has the following
/// API:
///
/// ```c
/// typedef struct foo foo_t;
///
/// extern foo_t *foo_new();
/// extern void foo_delete(foo_t*);
/// extern void foo_use(foo_t*);
/// ```
///
/// the bindings would look as follows
///
/// ```
/// pub struct Foo(cffi::Private);
///
/// impl Foo {
///     fn new() -> cffi::Ptr<Foo> {
///         Ptr::new(unsafe { foo_new() })
///     }
///
///     fn use(&mut self) {
///         unsafe { foo_use(self) }
///     }
/// }
///
/// impl cffi::Alloc for Foo {
///     fn free(this: *mut Self) {
///         unsafe { foo_delete(this) }
///     }
/// }
///
/// extern "C" {
///     fn foo_new() -> *mut Foo;
///     fn foo_delete(foo: *mut Foo);
///     fn foo_use(foo: *mut Foo);
/// }
/// ```
#[repr(C)]
pub struct Private(Never);

// TODO: replace with `!` once it's stabilized.
enum Never {}

/// Trait for FFI types which come with their own memory management.
pub trait Alloc {
    fn free(this: *mut Self);
}

/// Owned pointer.
///
/// This type is very similar to [`Box`] in that it is essentially an owned
/// pointer. The difference between them is that [`Box`] manages memory
/// allocation itself, while `Ptr` delegates this to the pointee's [`Alloc`]
/// implementation.
pub struct Ptr<T: Alloc>(*mut T);

impl<T: Alloc> Ptr<T> {
    pub unsafe fn from_raw(raw: *mut T) -> Ptr<T> {
        Ptr(raw)
    }

    pub fn into_raw(ptr: Ptr<T>) -> *mut T {
        let raw = ptr.0;
        mem::forget(ptr);
        raw
    }

    pub fn as_ptr(ptr: &Ptr<T>) -> *const T {
        ptr.0
    }

    pub fn as_raw(ptr: &mut Ptr<T>) -> *mut T {
        ptr.0
    }
}

impl<T: Alloc> AsRef<T> for Ptr<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T: Alloc> AsMut<T> for Ptr<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: Alloc> Borrow<T> for Ptr<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: Alloc> BorrowMut<T> for Ptr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: Alloc> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { mem::transmute(self.0) }
    }
}

impl<T: Alloc> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(self.0) }
    }
}

impl<T: Alloc> Drop for Ptr<T> {
    fn drop(&mut self) {
        Alloc::free(self.0);
    }
}

#[macro_export]
macro_rules! impl_ptr {
    ($wrapper:ty, $type:ty) => {
        impl AsRef<$type> for $wrapper {
            fn as_ref(&self) -> &$type {
                self.0.as_ref()
            }
        }

        impl ::std::ops::Deref for $wrapper {
            type Target = $type;

            fn deref(&self) -> &$type {
                self.0.deref()
            }
        }

        impl ::std::ops::DerefMut for $wrapper {
            fn deref_mut(&mut self) -> &mut $type {
                self.0.deref_mut()
            }
        }
    };
}
