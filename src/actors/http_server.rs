use async_trait::async_trait;
use serde_json;
use tide::Request;
use xactor::*;

use super::data_holder::*;
use crate::messages::*;

pub struct HttpServer {
    pub data_holder_addr: Addr<DataHolder>,
}

impl HttpServer {
    pub fn new(data_holder_addr: Addr<DataHolder>) -> Self {
        Self { data_holder_addr }
    }
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
        let mut app = tide::with_state(self.data_holder_addr.clone());
        app.at("/tail/:n").get(get_indicators);
        app.listen(format!("127.0.0.1:{}", msg.0)).await.unwrap();
    }
}

async fn get_indicators(req: Request<Addr<DataHolder>>) -> tide::Result {
    let state = req.state();
    let n: usize = req.param("n").unwrap().parse().unwrap_or_default();
    let data: Vec<Indicators> = state.call(GetIndicators(n)).await?;
    Ok(serde_json::to_string(&data)?.into())
}
