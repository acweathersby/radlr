#![crate_type = "rlib"]
#![feature(const_eval_limit)]
#![const_eval_limit = "0"]
#![feature(new_uninit)]
#![feature(get_mut_unchecked)]
#![feature(scoped_threads)]
#![feature(core_intrinsics)]
#![feature(box_patterns)]
#![feature(map_first_last)]
#![allow(
    bad_style,
    dead_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens
)]

pub mod primitives;
pub mod runtime;
pub mod utf8; 

pub use lazy_static::lazy_static;

pub mod ascript;
pub mod bytecode;
pub mod debug;
pub mod grammar;
pub mod intermediate;

// Common utility functions

use std::num::NonZeroUsize;

/// Retrieve the number of threads that can be reasonably
/// run concurrently on the platform

pub fn get_num_of_available_threads() -> usize
{
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).unwrap())
        .get()
}
