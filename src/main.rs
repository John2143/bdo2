mod engine;

#[tokio::main]
async fn main() -> (){
    engine::main().await;
}
