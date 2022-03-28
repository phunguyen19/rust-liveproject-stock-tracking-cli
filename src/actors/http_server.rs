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
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: StartHttpServer) {
        println!("Start HTTP Server");
        let mut app = tide::new();
        app.at("/hello").get(hello);
        app.listen("127.0.0.1:8080").await.unwrap();
    }
}

async fn hello(_: Request<()>) -> tide::Result {
    Ok(format!("Hello world").into())
}
