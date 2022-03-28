use chrono::prelude::*;
use xactor::*;

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
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub series: Vec<f64>,
}

#[message]
#[derive(Debug, Clone)]
pub struct Indicators {
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub last_price: f64,
    pub pct_change: f64,
    pub period_min: f64,
    pub period_max: f64,
    pub sma: Vec<f64>,
}
