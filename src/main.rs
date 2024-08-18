use axum::routing::get;
use axum::Error;
use axum::{extract::Query, response::IntoResponse, Json, Router};
use dp_video_source::{download_webpage, extract_video_data, get_ytplayer_config};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
struct VideoQuery {
    url: Option<String>,
}

#[derive(Serialize)]
struct VideoInfo {
    url: String,
    title: String,
    mime_type: String,
    statuscode: String,
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/extract", get(extract));

    Ok(router.into())
}

async fn extract(Query(params): Query<VideoQuery>) -> impl IntoResponse {
    info!("extract_video_info");
    let response = match &params.url {
        Some(url) => {
            info!("url {url}");
            // Use builder pattern to set options
            let res_await = doExtract(url).await;
            let res = match res_await {
                Some(res) => Ok(Json(res).into_response()),
                None => Err("Error"),
            };
            res
        }
        None => Err("Error"),
    };
    response
}

async fn doExtract(url: &str) -> Option<Vec<VideoInfo>> {
    info!("url {url}");
    // Use builder pattern to set options

    let contents = download_webpage(url);
    let player_config = get_ytplayer_config(contents).unwrap();
    let parsed_config = json::parse(&player_config).unwrap();
    let video_info_res = extract_video_data(parsed_config).await;

    match video_info_res {
        Some(video_info) => {
            info!("video_info received");
            let videos: Vec<VideoInfo> = video_info
                .streaming_data
                .borrow()
                .iter()
                .map(|x| VideoInfo {
                    title: video_info.title.clone(),
                    url: x.url.clone(),
                    mime_type: x.mime_type.to_string().clone(),
                    statuscode: x.statuscode.to_string().clone(),
                })
                .collect();
            Some(videos)
        }
        None => None,
    }
}
