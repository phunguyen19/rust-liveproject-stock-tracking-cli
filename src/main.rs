use async_std::stream;
use async_trait::async_trait;
use chrono::prelude::*;
use clap::Parser;
use futures::future::join_all;
use futures::stream::StreamExt;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use yahoo_finance_api as yahoo;

#[cfg(test)]
mod test;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

/// A trait to provide a common interface for all signal calculations.
#[async_trait]
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
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

/// Struct to implement the stock signal
/// to calculate the price difference from history
struct PriceDifference {}

#[async_trait]
impl AsyncStockSignal for PriceDifference {
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

/// Struct to implement the stock signal
/// to calculate the min price from history
struct MinPrice {}

/// Implement stock signal trait to
/// calculate min price
#[async_trait]
impl AsyncStockSignal for MinPrice {
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

/// Struct to implement the stock signal
/// to calculate the max price from history
struct MaxPrice {}

/// Implement stock signal trait
/// to calculate max price from history
#[async_trait]
impl AsyncStockSignal for MaxPrice {
    type SignalType = f64;

    ///
    /// Find the maximum in a series of f64
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
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
#[async_trait]
impl AsyncStockSignal for WindowedSMA {
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

/// Retrieve data from a data source and extract the closing prices. Errors during download are mapped onto io::Errors as InvalidData.
async fn fetch_closing_data(
    symbol: &str,
    beginning: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();

    let response = provider
        .get_quote_history(symbol, *beginning, *end)
        .await
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

/// This functions will do 3 things:
/// - First, fetch the data
/// - Then, calculate all the signals
/// - Final, it print the output through stdout
async fn fetch_n_calculate_n_output(
    symbol: &str,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
) -> std::io::Result<()> {
    // Fetching data
    let closes = fetch_closing_data(&symbol, &from, &to).await?;

    // Check if fetched data is not empty
    if !closes.is_empty() {
        // Calculate all the signal
        let period_max: f64 = MaxPrice {}.calculate(&closes).await.unwrap();
        let period_min: f64 = MinPrice {}.calculate(&closes).await.unwrap();
        let last_price = *closes.last().unwrap_or(&0.0);
        let (_, pct_change) = PriceDifference {}
            .calculate(&closes)
            .await
            .unwrap_or((0.0, 0.0));
        let sma = WindowedSMA { window_size: 30 }
            .calculate(&closes)
            .await
            .unwrap_or_default();

        // Output the data through stdout
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

    Ok(())
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    // Reading CLI args input
    let opts = Args::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let symbols = opts.symbols;

    // Print the header of data as CSV format through stdout
    println!("period start,symbol,price,change %,min,max,30d avg");

    // Init the interval counter
    let mut interval = stream::interval(Duration::from_secs(5));

    // Run the interval
    while let Some(_) = interval.next().await {
        // Init the current moment
        let to = Utc::now();
        // Create the vector to store the result form async call functions
        let mut futures = vec![];
        // Loop loop through the symbols
        // and call the async function of calculation for each symbol
        for symbol in symbols.split(',') {
            futures.push(fetch_n_calculate_n_output(symbol, &from, &to));
        }
        // Execute async functions
        join_all(futures).await;
    }

    Ok(())
}
