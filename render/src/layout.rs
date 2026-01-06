use crate::Buffer;

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

impl Buffer {
    pub fn with_header() -> Self {
        Self::from(HTML_HEADER)
    }

    pub fn add_footer(&mut self) {
        self.push_str(HTML_FOOTER);
    }
}
