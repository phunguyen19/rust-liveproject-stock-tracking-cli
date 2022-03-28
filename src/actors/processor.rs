use super::messages::*;
use crate::signals::*;
use async_trait::async_trait;
use xactor::*;

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
