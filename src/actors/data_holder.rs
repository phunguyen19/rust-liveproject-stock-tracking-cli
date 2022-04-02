use async_trait::async_trait;
use std::collections::VecDeque;
use xactor::*;

use crate::messages::*;

#[derive(Clone)]
pub struct DataHolder {
    pub indicators_vec: VecDeque<Indicators>,
}

impl DataHolder {
    pub fn new() -> Self {
        Self {
            indicators_vec: VecDeque::new(),
        }
    }
}

#[async_trait]
impl Actor for DataHolder {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.subscribe::<Indicators>().await
    }
}

#[async_trait]
impl Handler<Indicators> for DataHolder {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Indicators) {
        self.indicators_vec.push_front(msg);
    }
}

#[async_trait]
impl Handler<GetIndicators> for DataHolder {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: GetIndicators) -> Vec<Indicators> {
        let n = msg.0;

        if n <= 0 {
            return vec![];
        }

        self.indicators_vec
            .iter()
            .take(n)
            .cloned()
            .collect::<Vec<Indicators>>()
    }
}
