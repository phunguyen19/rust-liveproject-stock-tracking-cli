use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::{FuturesUnordered, StreamExt};
use std::io::{Error, ErrorKind};
use xactor::*;
use yahoo_finance_api as yahoo;

use crate::messages::*;

pub struct Fetcher;

impl Fetcher {
    async fn fetch_data(symbol: String, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<f64> {
        let provider = yahoo::YahooConnector::new();

        let res = provider
            .get_quote_history(symbol.as_str(), from, to)
            .await
            .map_err(|_| Error::from(ErrorKind::InvalidData));

        let mut series: Vec<f64> = Vec::new();
        match res {
            Err(_) => series,
            Ok(response) => {
                let mut quotes = response
                    .quotes()
                    .map_err(|_| Error::from(ErrorKind::InvalidData))
                    .unwrap();

                if !quotes.is_empty() {
                    quotes.sort_by_cached_key(|k| k.timestamp);
                    series = quotes.iter().map(|q| q.adjclose as f64).collect();
                }

                series
            }
        }
    }
}

#[async_trait]
impl Actor for Fetcher {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.subscribe::<FetchQuotes>().await
    }
}

#[async_trait]
impl Handler<FetchQuotes> for Fetcher {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: FetchQuotes) {
        let mut cf: FuturesUnordered<_> = msg
            .symbols
            .iter()
            .map(|symbol| {
                let series = Fetcher::fetch_data(symbol.clone(), msg.from, msg.to);
                let mut quote = Quote {
                    symbol: symbol.clone(),
                    from: msg.from,
                    series: Vec::new(),
                };

                async {
                    quote.series = series.await;
                    let _ = Broker::from_registry().await.unwrap().publish(quote);
                }
            })
            .collect();

        while let Some(_) = cf.next().await {}
    }
}
