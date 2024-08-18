use axum::http::StatusCode;
use axum::routing::get;
use axum::{extract::Query, response::IntoResponse, Json, Router};
use serde::{Deserialize, Serialize};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[derive(Deserialize)]
struct VideoQuery {
    url: Option<String>,
}

#[derive(Serialize)]
struct VideoInfo {
    best_format_url: String,
    title: String,
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/extract", get(extract_video_info));

    Ok(router.into())
}

async fn extract_video_info(query: Query<VideoQuery>) -> impl IntoResponse {
    match &query.url {
        Some(url) => {
            let output = YoutubeDl::new(url)
                .format("best")
                .run()
                .expect("Failed to run youtube-dl");

            let videos = match output {
                YoutubeDlOutput::SingleVideo(video) => vec![VideoInfo {
                    best_format_url: video.url.unwrap_or_default(),
                    title: video.title,
                }],
                YoutubeDlOutput::Playlist(playlist) => playlist
                    .entries
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|entry| {
                        Some(VideoInfo {
                            best_format_url: entry.url?,
                            title: entry.title,
                        })
                    })
                    .collect(),
            };

            Json(videos).into_response()
        }
        None => (
            StatusCode::BAD_REQUEST,
            "Please provide a URL as a query parameter.",
        )
            .into_response(),
    }
}
