pub mod engines;
pub mod indicators;
pub mod math;
pub mod model;
pub mod oi_trend;
pub mod option;
pub mod patterns;

pub use model::*;

/// Compute one indicator by id for the given candles and settings.
pub fn compute(id: &str, candles: &[Candle], settings: &Settings) -> Vec<SeriesOut> {
    match indicators::find(id) {
        Some(entry) => (entry.compute)(candles, settings),
        None => Vec::new(),
    }
}

/// Realtime helper: recompute only the last point of every series where possible.
/// For correctness this simply recomputes the whole series (the web engine only
/// calls it on ticks); callers can diff if they need speed.
pub fn recompute(id: &str, candles: &[Candle], settings: &Settings) -> Vec<SeriesOut> {
    compute(id, candles, settings)
}
