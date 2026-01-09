use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::{
    TypedHeader,
    extract::cookie::{Cookie, CookieJar, SameSite},
};
use headers_accept::Accept;
use mediatype::{
    MediaType,
    names::{HTML, TEXT},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uniremote_core::{CallActionRequest, RemoteId};
use uniremote_render::{Buffer, RenderHtml};

use crate::AppState;

const AUTH_COOKIE_NAME: &str = "uniremote_auth";

const CONTENT_TYPE_HTML: MediaType = MediaType::from_parts(TEXT, HTML, None, &[]);

pub async fn login(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    // Validate the token
    state.auth_token.validate(&token)?;
    
    // Set HTTP-only cookie with auth token
    let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
        .http_only(true)
        .path("/")
        .same_site(SameSite::Strict)
        .build();
    
    Ok((jar.add(cookie), Redirect::to("/")))
}

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
    html.push_str(r#"<h1>Available Remotes</h1><ul class="remote-list">"#);

    let mut remotes: Vec<_> = state
        .remotes
        .iter()
        .map(|(id, rwc)| (id, &rwc.remote))
        .collect();
    remotes.sort_by(|a, b| a.1.meta.name.cmp(&b.1.meta.name));

    for (id, remote) in remotes {
        html.push_str(r#"<li><a href="/r/"#);
        html.push_uri(id);
        html.push_str(r#""><img class="remote-icon" src="/r/"#);
        html.push_uri(id);
        html.push_str(r#"/icon" alt=""><div>"#);
        html.push_html(&remote.meta.name);
        html.push_str(r#"</div></a></li>"#);
    }

    html.push_str("</ul>");
    html.add_footer();

    html.into_response()
}

fn list_remotes_json(state: &AppState) -> Response {
    let mut remotes: Vec<_> = state
        .remotes
        .iter()
        .map(|(id, rwc)| (id, &rwc.remote))
        .collect();
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
    let remote_with_channel = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;
    let remote = &remote_with_channel.remote;

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
    jar: CookieJar,
    Json(request): Json<CallActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract token from cookie
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|cookie| cookie.value())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state.auth_token.validate(token)?;

    let remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!("call action '{}' on remote '{remote_id}'", request.action);

    if let Err(error) = remote.worker.send(request).await {
        tracing::error!("failed to send action request to worker: {error:#}");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(Json(serde_json::json!({
        "status": "pending",
    })))
}

pub async fn get_remote_icon(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    let remote_with_channel = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;
    let remote = &remote_with_channel.remote;

    // Use the resolved icon path from RemoteMeta
    let icon_path = remote.meta.resolve_icon_path(&remote.path);

    let (file_path, mime_type) = match icon_path {
        Some(path) => {
            let mime = path
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| match ext.to_lowercase().as_str() {
                    "png" => Some("image/png"),
                    "jpg" | "jpeg" => Some("image/jpeg"),
                    "gif" => Some("image/gif"),
                    "svg" => Some("image/svg+xml"),
                    "webp" => Some("image/webp"),
                    "ico" => Some("image/x-icon"),
                    _ => None,
                })
                .unwrap_or("application/octet-stream");
            (path, mime)
        }
        None => {
            // Fallback to default icon
            ("server/assets/gears.png".into(), "image/png")
        }
    };

    let file = File::open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .body(body)
        .unwrap())
}
