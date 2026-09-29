//! Fixed Kerr background, signature (-,+,+,+), G=c=M=1.
//! Physics has no dependency on UI, rendering, or a compute backend.
pub mod compute;
pub mod config;
pub mod experiments;
pub mod physics;

pub mod detector;

pub mod output;
pub mod session;
pub mod simulation;
pub mod standard_run;
pub mod visualization;
