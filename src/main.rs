mod actors;
mod signals;
#[cfg(test)]
mod test;

use std::time::Duration;

use actors::*;
use async_std::prelude::*;
use async_std::stream;
use chrono::prelude::*;
use clap::Parser;
use signals::*;
use xactor::*;

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

    let _fetcher_addr = Fetcher {}.start().await;
    let _processor_addr = Processor {}.start().await;
    let _writer_addr = Writer::new(output_file_name).start().await;

    let mut interval = stream::interval(Duration::from_secs(30));

    while let Some(_) = interval.next().await {
        // Init the current moment
        let to = Utc::now();
        // Loop loop through the symbols
        // and call the async function of calculation for each symbol
        let fetch_quote = FetchQuotes {
            symbols: symbols.clone(),
            from,
            to,
        };
        let _ = Broker::from_registry().await.unwrap().publish(fetch_quote);
    }

    Ok(())
}
