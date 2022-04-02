use async_std::fs::{File, OpenOptions};
use async_std::io::WriteExt;
use async_trait::async_trait;
use xactor::*;

use crate::messages::*;

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
                msg.last_sma,
            );

            let _ = file.write(s.as_bytes()).await;
        }
    }
}
