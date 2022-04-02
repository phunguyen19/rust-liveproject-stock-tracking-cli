use async_trait::async_trait;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use xactor::*;

use crate::messages::*;

#[derive(Clone)]
pub struct DataHolder {
    pub indicators_vec: Arc<Mutex<VecDeque<Indicators>>>,
}

impl DataHolder {
    pub fn new() -> Self {
        Self {
            indicators_vec: Arc::new(Mutex::new(VecDeque::new())),
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
        let mut indicators_vec = self.indicators_vec.lock().unwrap();
        indicators_vec.push_front(msg);
    }
}
