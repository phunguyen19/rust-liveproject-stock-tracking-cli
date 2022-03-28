use crate::*;

use async_std::{
    fs::{File, OpenOptions},
    io::WriteExt,
};
use async_trait::async_trait;
use chrono::prelude::*;
use futures::stream::FuturesUnordered;
use std::io::{Error, ErrorKind};
use xactor::*;
use yahoo_finance_api as yahoo;

#[message]
#[derive(Clone)]
pub struct FetchQuotes {
    pub symbols: Vec<String>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[message]
#[derive(Clone)]
pub struct Quote {
    symbol: String,
    from: DateTime<Utc>,
    series: Vec<f64>,
}

#[message]
#[derive(Debug, Clone)]
pub struct Indicators {
    symbol: String,
    from: DateTime<Utc>,
    last_price: f64,
    pct_change: f64,
    period_min: f64,
    period_max: f64,
    sma: Vec<f64>,
}

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
        let mut cf = msg
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
            .collect::<FuturesUnordered<_>>();

        while let Some(_) = cf.next().await {}
    }
}

pub struct Processor;

#[async_trait]
impl Actor for Processor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("period start,symbol,price,change %,min,max,30d avg");
        ctx.subscribe::<Quote>().await
    }
}

#[async_trait]
impl Handler<Quote> for Processor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Quote) {
        if msg.series.is_empty() {
            return ();
        }

        let period_max: f64 = MaxPrice {}.calculate(&msg.series).await.unwrap();
        let period_min: f64 = MinPrice {}.calculate(&msg.series).await.unwrap();
        let last_price = *msg.series.last().unwrap_or(&0.0);
        let (_, pct_change) = PriceDifference {}
            .calculate(&msg.series)
            .await
            .unwrap_or((0.0, 0.0));
        let sma = WindowedSMA { window_size: 30 }
            .calculate(&msg.series)
            .await
            .unwrap_or_default();

        let indicators = Indicators {
            symbol: msg.symbol.clone(),
            from: msg.from,
            last_price,
            pct_change,
            period_min,
            period_max,
            sma: sma.clone(),
        };

        let _ = Broker::from_registry().await.unwrap().publish(indicators);

        println!(
            "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            msg.from.to_rfc3339(),
            msg.symbol,
            last_price,
            pct_change * 100.0,
            period_min,
            period_max,
            sma.last().unwrap_or(&0.0)
        );
    }
}

pub struct Writer {
    filename: String,
    file: Option<File>,
}

impl Writer {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            file: None,
        }
    }
}

#[async_trait]
impl Actor for Writer {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.filename.clone())
            .await
            .unwrap();

        let header = b"period start,symbol,price,change %,min,max,30d avg";
        let _ = file.write(header).await;
        self.file = Some(file);
        ctx.subscribe::<Indicators>().await
    }
}

#[async_trait]
impl Handler<Indicators> for Writer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Indicators) {
        if let Some(mut file) = self.file.clone() {
            let s = format!(
                "\n{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                msg.from.to_rfc3339(),
                msg.symbol,
                msg.last_price,
                msg.pct_change * 100.0,
                msg.period_min,
                msg.period_max,
                msg.sma.last().unwrap_or(&0.0),
            );

            let _ = file.write(s.as_bytes()).await;
        }
    }
}
