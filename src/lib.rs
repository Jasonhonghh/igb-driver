#![no_std]

extern crate alloc;

mod descriptor;
mod err;
mod igb;
mod phy;
mod regs;
mod ring;

use core::time::Duration;

pub use igb::*;

pub use dma_api::Impl;

pub trait Kernel {
    fn sleep(duration: Duration);
    fn phy_to_vir(addr:u64)->u64;
}

pub(crate) fn sleep(duration: Duration) {
    extern "Rust" {
        fn _igb_driver_sleep(duration: Duration);
    }

    unsafe {
        _igb_driver_sleep(duration);
    }
}
pub(crate) fn phy_to_vir(addr:u64) ->u64 {
    extern "Rust" {
        fn _igb_driver_phy_to_vir(addr:u64) ->u64 ;
    }

    unsafe {
        _igb_driver_phy_to_vir(addr)
    }
}

#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _igb_driver_sleep(duration: core::time::Duration) {
            <$t as $crate::Kernel>::sleep(duration)
        }
    };
}
#[macro_export]
macro_rules! set_impl1 {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _igb_driver_phy_to_vir(addr:u64) ->u64 {
            <$t as $crate::Kernel>:: phy_to_vir(addr)
        }
    };
}
