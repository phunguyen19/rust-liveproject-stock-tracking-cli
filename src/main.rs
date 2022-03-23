use chrono::prelude::*;
use clap::Clap;
use std::io::{Error, ErrorKind};
use yahoo_finance_api as yahoo;

#[cfg(test)]
mod test;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Claus Matzinger",
    about = "A Manning LiveProject: async Rust"
)]
struct Args {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

/// Share behaviors to calculate the
/// signal of stock based on the history price
trait StockSignal {
    /// The signal result after calculation
    type SignalType;

    /// Calculate the signal base on history price
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

/// Struct to implement the stock signal
/// to calculate the price difference from history
struct PriceDifference {}

impl StockSignal for PriceDifference {
    /// Stock signal for price difference
    /// contains 2 numbers:
    /// - absolute difference
    /// - relative difference
    type SignalType = (f64, f64);

    ///
    /// Calculates the absolute and relative difference between the beginning and ending of an f64 series. The relative difference is relative to the beginning.
    ///
    /// # Returns
    ///
    /// A tuple `(absolute, relative)` difference.
    ///
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
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

/// Struct to implement the stock signal
/// to calculate the min price from history
struct MinPrice {}

/// Implement stock signal trait to
/// calculate min price
impl StockSignal for MinPrice {
    type SignalType = f64;

    ///
    /// Find the minimum in a series of f64
    ///
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }
}

/// Struct to implement the stock signal
/// to calculate the max price from history
struct MaxPrice {}

/// Implement stock signal trait
/// to calculate max price from history
impl StockSignal for MaxPrice {
    type SignalType = f64;

    ///
    /// Find the maximum in a series of f64
    ///
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }
}

/// Struct to implement data and behaviros
/// to calculate Simple Moving Average
struct WindowedSMA {
    window_size: usize,
}

/// Implement stock signal to
/// calculate Simple Moving Average
impl StockSignal for WindowedSMA {
    type SignalType = Vec<f64>;

    ///
    /// Find the maximum in a series of f64
    ///
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
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

///
/// A trait to provide a common interface for all signal calculations.
///
trait AsyncStockSignal {
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
    fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

///
/// Retrieve data from a data source and extract the closing prices. Errors during download are mapped onto io::Errors as InvalidData.
///
fn fetch_closing_data(
    symbol: &str,
    beginning: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();

    let response = provider
        .get_quote_history(symbol, *beginning, *end)
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let mut quotes = response
        .quotes()
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    if !quotes.is_empty() {
        quotes.sort_by_cached_key(|k| k.timestamp);
        Ok(quotes.iter().map(|q| q.adjclose as f64).collect())
    } else {
        Ok(vec![])
    }
}

fn main() -> std::io::Result<()> {
    let opts = Args::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let to = Utc::now();

    // a simple way to output a CSV header
    println!("period start,symbol,price,change %,min,max,30d avg");
    for symbol in opts.symbols.split(',') {
        let closes = fetch_closing_data(&symbol, &from, &to)?;

        if !closes.is_empty() {
            // min/max of the period. unwrap() because those are Option types
            let period_max: f64 = MaxPrice {}.calculate(&closes).unwrap();
            let period_min: f64 = MinPrice {}.calculate(&closes).unwrap();
            let last_price = *closes.last().unwrap_or(&0.0);
            let (_, pct_change) = PriceDifference {}.calculate(&closes).unwrap_or((0.0, 0.0));
            let sma = WindowedSMA { window_size: 30 }
                .calculate(&closes)
                .unwrap_or_default();

            // a simple way to output CSV data
            println!(
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                from.to_rfc3339(),
                symbol,
                last_price,
                pct_change * 100.0,
                period_min,
                period_max,
                sma.last().unwrap_or(&0.0)
            );
        }
    }
    Ok(())
}
