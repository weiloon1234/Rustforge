use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn serve(router: Router, bind_addr: SocketAddr) -> anyhow::Result<()> {
    tracing::info!("Listening on http://{bind_addr}");
    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
