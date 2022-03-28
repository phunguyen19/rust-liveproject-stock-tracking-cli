mod actors;
mod signals;
#[cfg(test)]
mod test;

use std::time::Duration;

use async_std::prelude::*;
use async_std::stream;
use chrono::prelude::*;
use clap::Parser;
use xactor::*;

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

#[xactor::main]
async fn main() -> std::io::Result<()> {
    // Reading CLI args input
    let opts = Args::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let symbols: Vec<String> = opts
        .symbols
        .split(',')
        .map(|symbol| String::from(symbol))
        .collect();

    let output_file_name = format!("{}.csv", Utc::now().to_rfc2822());

    let _fetcher_addr = actors::fetcher::Fetcher {}.start().await;
    let _processor_addr = actors::processor::Processor {}.start().await;
    let _writer_addr = actors::writer::Writer::new(output_file_name).start().await;
    let _http_server_addr = actors::http_server::HttpServer {}.start().await;

    let _ = Broker::from_registry()
        .await
        .unwrap()
        .publish(actors::messages::StartHttpServer);

    let mut interval = stream::interval(Duration::from_secs(5));
    while let Some(_) = interval.next().await {
        // Init the current moment
        let to = Utc::now();
        // Loop loop through the symbols
        // and call the async function of calculation for each symbol
        let fetch_quote = actors::messages::FetchQuotes {
            symbols: symbols.clone(),
            from,
            to,
        };
        let _ = Broker::from_registry().await.unwrap().publish(fetch_quote);
    }

    Ok(())
}
