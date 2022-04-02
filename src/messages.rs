use chrono::prelude::*;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Indicators {
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub last_price: f64,
    pub pct_change: f64,
    pub period_min: f64,
    pub period_max: f64,
    pub last_sma: f64,
}
#[message]
#[derive(Debug, Clone)]
pub struct StartHttpServer(pub u32);

#[message(result = "Vec<Indicators>")]
#[derive(Debug, Clone)]
pub struct GetIndicators(pub usize);
