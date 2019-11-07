#![feature(in_band_lifetimes)]
#![feature(generators, cell_update)]
#![allow(unused_imports, dead_code)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

pub mod market;
pub mod goods;
pub mod agent;
pub mod record;




