/* Copyright (c) Fortanix, Inc.
 *
 * Licensed under the GNU General Public License, version 2 <LICENSE-GPL or
 * https://www.gnu.org/licenses/gpl-2.0.html> or the Apache License, Version
 * 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>, at your
 * option. This file may not be copied, modified, or distributed except
 * according to those terms. */

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
extern crate core;

pub mod types;

#[cfg(feature = "rust-bindgen")]
include!(concat!(env!("OUT_DIR"), "/mod-bindings.rs"));

#[cfg(not(feature = "rust-bindgen"))]
include!("../bindings/mod-bindings.rs");

pub use bindings::*;
