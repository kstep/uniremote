use std::ops::Deref;

use axum::response::{Html, IntoResponse, Response};

const DEFAULT_BUFFER_SIZE: usize = 1024;

pub struct Buffer {
    content: String,
}

impl Deref for Buffer {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            content: String::with_capacity(DEFAULT_BUFFER_SIZE),
        }
    }
}

impl<T: AsRef<str>> From<T> for Buffer {
    fn from(s: T) -> Self {
        Self {
            content: s.as_ref().to_string(),
        }
    }
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.content.push_str(s);
    }

    pub fn push_html(&mut self, s: &str) {
        html_escape::encode_safe_to_string(s, &mut self.content);
    }

    pub fn push_uri(&mut self, s: &str) {
        let encoded = uri_encode::encode_uri_component(s);
        self.content.push_str(&encoded);
    }

    pub fn push_url(&mut self, s: &str) {
        let encoded = uri_encode::encode_uri(s);
        self.content.push_str(&encoded);
    }

    pub fn push_char(&mut self, c: char) {
        self.content.push(c);
    }

    pub fn into_html(self) -> Html<String> {
        Html(self.content)
    }
}

impl Into<String> for Buffer {
    fn into(self) -> String {
        self.content
    }
}

impl IntoResponse for Buffer {
    fn into_response(self) -> Response {
        self.into_html().into_response()
    }
}
