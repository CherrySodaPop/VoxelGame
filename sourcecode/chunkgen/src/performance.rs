//! Stuff for performance testing.

use std::{fmt::Formatter, time::Duration};

/// Helper struct for timing chunk/mesh generation performance.
pub struct Timings {
    // TODO: Untie this from chunk/mesh stuff specifically
    pub generate_chunk: Vec<Duration>,
    pub build_mesh: Vec<Duration>,
}

impl Timings {
    pub fn new() -> Self {
        Self {
            generate_chunk: Vec::new(),
            build_mesh: Vec::new(),
        }
    }

    /// Returns the average length of the `Duration`s, in microseconds.
    fn average(of: &[Duration]) -> f64 {
        of.iter().map(|dur| dur.as_micros()).sum::<u128>() as f64 / of.len() as f64
    }
}

impl std::fmt::Display for Timings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gen_chunk_avg = Self::average(&self.generate_chunk) / 1000.0;
        let bld_mesh_avg = Self::average(&self.build_mesh) / 1000.0;
        write!(
            f,
            "-- Timings (avg) --\n\tChunk generation : {:.3} ms\n\tMesh building    : {:.3} ms",
            gen_chunk_avg, bld_mesh_avg
        )
    }
}
