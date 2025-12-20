use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::{self, HeaderValue},
    response::{Html, Redirect},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::templates::PAGE;

#[derive(Deserialize, Serialize)]
struct Member {
    id: String,
    name: String,
    url: String,
}
#[derive(Deserialize, Serialize)]
struct WebRing(Vec<Member>);

impl WebRing {
    fn position(&self, id: &str) -> Option<usize> {
        self.0.iter().position(|x| x.id == id)
    }
}

fn get_webring() -> WebRing {
    serde_json::from_str(include_str!("../webring.json")).unwrap()
}

async fn next(Path(id): Path<String>, State(ring): State<Arc<WebRing>>) -> Redirect {
    if let Some(idx) = ring.position(&id) {
        let next = idx
            .checked_add(1)
            .filter(|&i| i < ring.0.len())
            .unwrap_or(0);
        Redirect::to(&ring.0[next].url)
    } else {
        Redirect::to("/")
    }
}

async fn prev(Path(id): Path<String>, State(ring): State<Arc<WebRing>>) -> Redirect {
    if let Some(idx) = ring.position(&id) {
        let prev = idx.wrapping_sub(1).min(ring.0.len() - 1);
        Redirect::to(&ring.0[prev].url)
    } else {
        Redirect::to("/")
    }
}

async fn page() -> Html<String> {
    Html(PAGE.clone())
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
        .route("/", get(page))
        .route("/next/{idx}", get(next))
        .route("/prev/{idx}", get(prev))
        .with_state(Arc::new(webring))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod templates {
    use lazy_static::lazy_static;
    use tera::{Context, Tera};

    use crate::get_webring;

    lazy_static! {
        pub static ref PAGE: String = {
            let tera = match Tera::new("src/templates/*.html") {
                Ok(t) => t,
                Err(e) => {
                    println!("Parsing error(s): {}", e);
                    ::std::process::exit(1);
                }
            };
            let mut context = Context::new();
            context.insert("members", &get_webring());
            tera.render("index.html", &context).unwrap()
        };
    }
}
