#![no_std]
use core::{
    mem::ManuallyDrop,
    task::{RawWaker, RawWakerVTable},
};

use externref::{Resource, externref};
macro_rules! resource {
    ($name:ident) => {
        #[repr(transparent)]
        pub struct $name(pub Resource<$name>);
    };
}
resource!(Waker);

const _: () = {
    mod ffi {
        use super::*;
        #[externref]
        #[link(wasm_import_module = "waker")]
        unsafe extern "C" {

            pub fn wake(w: &Resource<Waker>);
            pub fn clone_waker(w: &Resource<Waker>) -> Resource<Waker>;
            pub fn new_waker(p0: *const (), p1: &'static RawWakerVTable) -> Resource<Waker>;
        }
    }
    pub fn do_wake(a: Resource<Waker>) -> RawWaker {
        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |a| unsafe {
                do_wake(ffi::clone_waker(&*core::mem::transmute::<
                    _,
                    ManuallyDrop<Resource<Waker>>,
                >(a)))
            },
            |a| unsafe {
                ffi::wake(&core::mem::transmute::<_, Resource<Waker>>(a));
            },
            |a| unsafe {
                ffi::wake(&*core::mem::transmute::<_, ManuallyDrop<Resource<Waker>>>(
                    a,
                ))
            },
            |a| unsafe {
                core::mem::transmute::<_, Resource<Waker>>(a);
            },
        );
        RawWaker::new(unsafe { core::mem::transmute(a) }, &VTABLE)
    }
    pub fn waker_of(a: Resource<Waker>) -> core::task::Waker {
        unsafe { core::task::Waker::from_raw(do_wake(a)) }
    }
    pub fn resource_of(a: core::task::Waker) -> Resource<Waker> {
        unsafe { ffi::new_waker(a.data(), a.vtable()) }
    }

    #[unsafe(export_name = "waker/wake")]
    extern "C" fn wake(p0: *const (), p1: &'static RawWakerVTable) {
        let w = unsafe { core::task::Waker::new(p0, p1) };
        w.wake();
    }
    #[unsafe(export_name = "waker/wake_by_ref")]
    extern "C" fn wake_by_ref(p0: *const (), p1: &'static RawWakerVTable) {
        let w = unsafe { ManuallyDrop::new(core::task::Waker::new(p0, p1)) };
        w.wake_by_ref();
    }
    #[unsafe(export_name = "waker/drop")]
    extern "C" fn drop(p0: *const (), p1: &'static RawWakerVTable) {
        let w = unsafe { core::task::Waker::new(p0, p1) };
    }
    #[externref]
    #[unsafe(export_name = "waker/clone")]
    extern "C" fn clone(p0: *const (), p1: &'static RawWakerVTable) -> Resource<Waker> {
        let w = unsafe { ManuallyDrop::new(core::task::Waker::new(p0, p1)) };
        resource_of((&*w).clone())
    }
    impl From<core::task::Waker> for Waker {
        fn from(value: core::task::Waker) -> Self {
            Waker(resource_of(value))
        }
    }
    impl From<Waker> for core::task::Waker {
        fn from(value: Waker) -> Self {
            waker_of(value.0)
        }
    }
    impl Clone for Waker {
        fn clone(&self) -> Self {
            Self(unsafe { ffi::clone_waker(&self.0) })
        }
    }
};
