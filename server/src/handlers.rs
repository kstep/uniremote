use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers_accept::Accept;
use mediatype::{
    MediaType,
    names::{HTML, TEXT},
};

use crate::AppState;

const CONTENT_TYPE_HTML: MediaType = MediaType::from_parts(TEXT, HTML, None, &[]);

static HTML_HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>UniRemote</title>
</head>
<body>
"#;

static HTML_FOOTER: &str = r#"</body></html>"#;

pub async fn list_remotes(
    State(state): State<Arc<AppState>>,
    accept: Option<TypedHeader<Accept>>,
) -> Response {
    let wants_html = accept.as_ref().is_some_and(|TypedHeader(accept)| {
        accept
            .media_types()
            .any(|mime| mime.essence() == CONTENT_TYPE_HTML)
    });

    if wants_html {
        list_remotes_html(&state)
    } else {
        list_remotes_json(&state)
    }
}

fn list_remotes_html(state: &AppState) -> Response {
    let mut html = String::from(HTML_HEADER);
    html.push_str(r#"<h1>Available Remotes</h1><ul>"#);

    for (id, remote) in &state.remotes {
        html.push_str(&format!(
            r#"<li><a href="/r/{id}">{}</a></li>"#,
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
