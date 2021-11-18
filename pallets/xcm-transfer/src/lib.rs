#![cfg_attr(not(feature = "std"), no_std)]

pub mod common;
pub mod xcm_helper;
pub mod xcm_transfer;

// Alias
pub use xcm_transfer as pallet_xcm_transfer;

#[cfg(test)]
mod mock;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;
