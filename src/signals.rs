use async_trait::async_trait;
use clap::Parser;

///
/// Struct to store arguments from command
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    ///
    /// Stock symbols. E.g: AAPL,MSFT,UBER,GOOG
    ///
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    pub symbols: String,

    ///
    /// Date in the past to start fetching prices
    ///
    #[clap(short, long)]
    pub from: String,
}

///
/// A trait to provide a common interface for all signal calculations.
///
#[async_trait]
pub trait AsyncStockSignal {
    ///
    /// The signal's data type.
    ///
    type SignalType;

    ///
    /// Calculate the signal on the provided series.
    ///
    /// # Returns
    ///
    /// The signal (using the provided type) or `None` on error/invalid data.
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

///
/// Struct to implement the stock signal
/// to calculate the price difference from history
///
pub struct PriceDifference {}

#[async_trait]
impl AsyncStockSignal for PriceDifference {
    ///
    /// Stock signal for price difference contains 2 numbers:
    /// - absolute difference
    /// - relative difference
    ///
    type SignalType = (f64, f64);

    ///
    /// Calculates the absolute and relative difference between the beginning and ending of an f64 series.
    /// The relative difference is relative to the beginning.
    ///
    /// # Returns
    ///
    /// A tuple `(absolute, relative)` difference.
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if !series.is_empty() {
            // unwrap is safe here even if first == last
            let (first, last) = (series.first().unwrap(), series.last().unwrap());
            let abs_diff = last - first;
            let first = if *first == 0.0 { 1.0 } else { *first };
            let rel_diff = abs_diff / first;
            Some((abs_diff, rel_diff))
        } else {
            None
        }
    }
}

///
/// Struct to implement the stock signal
/// to calculate the min price from history
///
pub struct MinPrice {}

#[async_trait]
impl AsyncStockSignal for MinPrice {
    ///
    /// Stock signal min price in the series of prices
    ///
    type SignalType = f64;

    ///
    /// Find the minimum in a series of f64
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }
}

///
/// Struct to implement the stock signal
/// to calculate the max price from history
///
pub struct MaxPrice {}

#[async_trait]
impl AsyncStockSignal for MaxPrice {
    ///
    /// Max price in the series of prices
    ///
    type SignalType = f64;

    ///
    /// Find the maximum in a series of prices
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }
}

///
/// Struct to implement data and behavior
/// to calculate Simple Moving Average
///
pub struct WindowedSMA {
    ///
    /// Number of day for calculation price
    ///
    pub window_size: usize,
}

#[async_trait]
impl AsyncStockSignal for WindowedSMA {
    ///
    /// Signal of Simple Moving Average
    ///
    type SignalType = Vec<f64>;

    ///
    /// Find the maximum in a series of f64
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        let n = self.window_size;
        if !series.is_empty() && n > 1 {
            Some(
                series
                    .windows(n)
                    .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                    .collect(),
            )
        } else {
            None
        }
    }
}
