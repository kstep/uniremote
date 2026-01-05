use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers_accept::Accept;
use mediatype::{
    MediaType,
    names::{HTML, TEXT},
};
use uniremote_core::{CallActionRequest, RemoteId};

use crate::AppState;

const CONTENT_TYPE_HTML: MediaType = MediaType::from_parts(TEXT, HTML, None, &[]);

static HTML_HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>UniRemote</title>
    <script src="/assets/frontend.js"></script>
    <link rel="stylesheet" href="/assets/style.css">
</head>
<body>
"#;

static HTML_FOOTER: &str = r#"</body></html>"#;

pub async fn list_remotes(
    State(state): State<Arc<AppState>>,
    query: Query<crate::auth::TokenQuery>,
    accept: Option<TypedHeader<Accept>>,
) -> Result<Response, StatusCode> {
    // Validate token
    crate::auth::validate_token(&query, &state)?;

    let wants_html = accept.as_ref().is_some_and(|TypedHeader(accept)| {
        accept
            .media_types()
            .any(|mime| mime.essence() == CONTENT_TYPE_HTML)
    });

    let token = query.token.as_deref().unwrap_or("");

    if wants_html {
        Ok(list_remotes_html(&state, token))
    } else {
        Ok(list_remotes_json(&state))
    }
}

fn list_remotes_html(state: &AppState, token: &str) -> Response {
    let mut html = String::from(HTML_HEADER);
    html.push_str(r#"<h1>Available Remotes</h1><ul>"#);

    for (id, remote) in &state.remotes {
        html.push_str(&format!(
            r#"<li><a href="/r/{id}?token={}">{}</a></li>"#,
            urlencoding::encode(token),
            remote.meta.name
        ));
    }

    html.push_str("</ul>");
    html.push_str(HTML_FOOTER);

    Html(html).into_response()
}

fn list_remotes_json(state: &AppState) -> Response {
    let remotes: Vec<_> = state
        .remotes
        .iter()
        .map(|(id, remote)| {
            serde_json::json!({
                "id": &*id,
                "name": remote.meta.name,
            })
        })
        .collect();

    Json(serde_json::json!({ "remotes": remotes })).into_response()
}

pub async fn get_remote(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    query: Query<crate::auth::TokenQuery>,
) -> Result<Html<String>, StatusCode> {
    // Validate token
    crate::auth::validate_token(&query, &state)?;

    let remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    let token = query.token.as_deref().unwrap_or("");
    let mut output = String::from(HTML_HEADER);

    output.push_str(&format!(
        "<div class=\"backlink\"><a href=\"/?token={}\">&larr; Back to remotes</a></div>",
        urlencoding::encode(token)
    ));
    output.push_str("<h1>");
    output.push_str(&remote.meta.name);
    output.push_str("</h1>");

    uniremote_render::render_layout(&mut output, &remote.layout);
    output.push_str(HTML_FOOTER);
    Ok(Html(output))
}

pub async fn call_remote_action(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    query: Query<crate::auth::TokenQuery>,
    Json(payload): Json<CallActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate token
    crate::auth::validate_token(&query, &state)?;

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
