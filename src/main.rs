use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::{self, HeaderValue},
    response::Redirect,
    routing::get,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct Member {
    name: String,
    url: String,
}
#[derive(Deserialize)]
struct WebRing(Vec<Member>);

fn get_webring() -> WebRing {
    serde_json::from_str(include_str!("../webring.json")).unwrap()
}

async fn next(Path(idx): Path<usize>, State(ring): State<Arc<WebRing>>) -> Redirect {
    let next = idx
        .checked_add(1)
        .filter(|&i| i < ring.0.len())
        .unwrap_or(0);
    Redirect::to(&ring.0[next].url)
}

async fn prev(Path(idx): Path<usize>, State(ring): State<Arc<WebRing>>) -> Redirect {
    let prev = idx.wrapping_sub(1).min(ring.0.len() - 1);
    Redirect::to(&ring.0[prev].url)
}

#[tokio::main]
async fn main() {
    let webring = get_webring();
    let allowed_origins = webring
        .0
        .iter()
        .map(|x| {
            x.url
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| panic!("url {} should be valid", x.url))
        })
        .collect::<Vec<_>>();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([http::Method::GET]);

    let app = Router::new()
        .route("/next/{idx}", get(next))
        .route("/prev/{idx}", get(prev))
        .with_state(Arc::new(webring))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
