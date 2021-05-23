pub mod algorithms;

#[cfg(feature = "log")]
#[macro_use]
extern crate log;

#[cfg(not(feature = "log"))]
#[macro_use]
pub mod log;

pub mod models;
pub mod remote;
pub mod simulator;
pub mod solver;
