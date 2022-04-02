use super::data_holder::*;
use super::messages::*;
use async_trait::async_trait;
use tide::Request;
use xactor::*;

pub struct HttpServer;

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
        let mut app = tide::with_state(DataHolder::new());
        app.at("/hello").get(hello);
        app.listen(format!("127.0.0.1:{}", msg.0)).await.unwrap();
    }
}

async fn hello(req: Request<DataHolder>) -> tide::Result {
    let state = req.state();
    let mut mutable_count = state.req_count.lock().unwrap();
    *mutable_count += 1;
    Ok(format!("Hello world {}", mutable_count).into())
}
