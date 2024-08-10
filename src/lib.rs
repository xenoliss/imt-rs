#![feature(trait_alias)]

#[cfg(feature = "circuits")]
pub mod circuits;

pub mod imt;

pub type Hash = [u8; 32];
