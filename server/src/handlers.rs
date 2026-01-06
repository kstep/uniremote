use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response, sse::{Event, Sse}},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use futures::stream::Stream;
use headers_accept::Accept;
use mediatype::{
    MediaType,
    names::{HTML, TEXT},
};
use std::convert::Infallible;
use uniremote_core::{CallActionRequest, RemoteId};
use uniremote_render::{Buffer, RenderHtml};

use crate::{AppState, auth::validate_token};

const CONTENT_TYPE_HTML: MediaType = MediaType::from_parts(TEXT, HTML, None, &[]);

pub async fn list_remotes(
    State(state): State<Arc<AppState>>,
    accept: Option<TypedHeader<Accept>>,
) -> Result<Response, StatusCode> {
    let wants_html = accept.as_ref().is_some_and(|TypedHeader(accept)| {
        accept
            .media_types()
            .any(|mime| mime.essence() == CONTENT_TYPE_HTML)
    });

    if wants_html {
        Ok(list_remotes_html(&state))
    } else {
        Ok(list_remotes_json(&state))
    }
}

fn list_remotes_html(state: &AppState) -> Response {
    let mut html = Buffer::with_header();
    html.push_str(r#"<h1>Available Remotes</h1><ul>"#);

    let mut remotes: Vec<_> = state.remotes.iter().collect();
    remotes.sort_by(|a, b| a.1.meta.name.cmp(&b.1.meta.name));

    for (id, remote) in remotes {
        html.push_str(r#"<li><a href="/r/"#);
        html.push_html(&id);
        html.push_str(r#"">"#);
        html.push_html(&remote.meta.name);
        html.push_str(r#"</a></li>"#);
    }

    html.push_str("</ul>");
    html.add_footer();

    html.into_response()
}

fn list_remotes_json(state: &AppState) -> Response {
    let mut remotes: Vec<_> = state.remotes.iter().collect();
    remotes.sort_by(|a, b| a.1.meta.name.cmp(&b.1.meta.name));

    let remotes: Vec<_> = remotes
        .into_iter()
        .map(|(id, remote)| {
            serde_json::json!({
                "id": id,
                "name": remote.meta.name,
            })
        })
        .collect();

    Json(serde_json::json!({ "remotes": remotes })).into_response()
}

pub async fn get_remote(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    let mut output = Buffer::with_header();

    output.push_str("<div class=\"backlink\"><a href=\"/\">&larr; Back to remotes</a></div><h1>");
    output.push_html(&remote.meta.name);
    output.push_str("</h1>");

    remote.layout.render(&mut output);
    output.add_footer();

    Ok(output.into_html())
}

pub async fn call_remote_action(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<CallActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    validate_token(auth_header, &state)?;

    let _remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!("call action '{}' on remote '{remote_id}'", payload.action);

    state
        .worker_tx
        .send((remote_id, payload))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "pending",
    })))
}

pub async fn sse_handler(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    validate_token(auth_header, &state)?;

    // Verify the remote exists
    let _remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!("SSE connection established for remote '{remote_id}'");

    // Subscribe to the broadcast channel
    let mut rx = state.sse_tx.subscribe();
    
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok((msg_remote_id, message)) => {
                    // Only send messages for this specific remote
                    if msg_remote_id == remote_id {
                        let json_str = match serde_json::to_string(&message) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("Failed to serialize SSE message: {}", e);
                                continue;
                            }
                        };
                        
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("SSE client lagged behind by {} messages", n);
                    // Continue processing, client will catch up
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::info!("SSE broadcast channel closed");
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    ))
}
