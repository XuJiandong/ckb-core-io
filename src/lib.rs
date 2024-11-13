#![doc = include_str!("../README.md")]
#![no_std]
#![allow(clippy::doc_lazy_continuation)]

extern crate alloc;

#[cfg(not(doctest))]
pub mod io;
