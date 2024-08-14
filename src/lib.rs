#![feature(trait_alias)]

#[cfg(feature = "circuits")]
pub mod circuits;

#[cfg(feature = "circuits")]
type Hash = [u8; 32];
