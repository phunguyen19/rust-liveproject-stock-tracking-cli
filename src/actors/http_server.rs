use async_trait::async_trait;
use serde_json;
use std::ops::Range;
use tide::Request;
use xactor::*;

use super::data_holder::*;
use crate::messages::*;

pub struct HttpServer {
    pub data_holder: DataHolder,
}

#[async_trait]
impl Actor for HttpServer {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.subscribe::<StartHttpServer>().await
    }
}

#[async_trait]
impl Handler<StartHttpServer> for HttpServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StartHttpServer) {
        println!("Start HTTP Server at {}", msg.0);
        let mut app = tide::with_state(self.data_holder.clone());
        app.at("/tail/:n").get(get_indicators);
        app.listen(format!("127.0.0.1:{}", msg.0)).await.unwrap();
    }
}

async fn get_indicators(req: Request<DataHolder>) -> tide::Result {
    let state = req.state();
    let n: usize = req.param("n").unwrap().parse().unwrap_or_default();
    let cloned = state.indicators_vec.clone();
    let indicators_vec = cloned.lock().unwrap();
    let mut data: Vec<Indicators> = vec![];

    if n <= 0 {
        return Ok(serde_json::to_string(&data)?.into());
    }

    if n > indicators_vec.len() {
        for item in indicators_vec.clone() {
            data.push(item.clone());
        }
    } else {
        for item in indicators_vec.range(Range { start: 0, end: n }) {
            data.push(item.clone());
        }
    }

    Ok(serde_json::to_string(&data)?.into())
}
