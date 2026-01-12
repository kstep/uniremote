use std::{str::FromStr, time::Duration};

use mlua::{Error, Function, Lua, Result, Table, Value};
use reqwest::{
    Method, RequestBuilder, Response,
    header::{HeaderMap, HeaderName, HeaderValue},
};

async fn get(lua: Lua, (url, callback): (String, Option<Function>)) -> Result<Value> {
    let client = create_client()?;
    let request = client.get(&url);

    request_internal(lua, request, callback).await
}

async fn post(
    lua: Lua,
    (url, data, callback): (String, Option<String>, Option<Function>),
) -> Result<Value> {
    let client = create_client()?;
    let mut request = client.post(&url);

    if let Some(body) = data {
        request = request.body(body);
    }

    request_internal(lua, request, callback).await
}

async fn request(lua: Lua, (request_table, callback): (Table, Option<Function>)) -> Result<Value> {
    let method = request_table.get::<String>("method").and_then(|m| {
        m.parse::<Method>()
            .map_err(|_| Error::runtime("invalid method"))
    })?;

    let url = request_table.get::<String>("url")?;

    let content: Option<String> = request_table.get("content").ok();
    let mime: Option<String> = request_table.get("mime").ok();

    let headers = request_table
        .get::<Table>("headers")
        .map(|headers| {
            headers
                .pairs::<String, String>()
                .filter_map(Result::ok)
                .filter_map(|(name, value)| {
                    HeaderName::from_str(&name)
                        .ok()
                        .zip(HeaderValue::from_str(&value).ok())
                })
                .collect::<HeaderMap>()
        })
        .unwrap_or_default();

    let client = create_client()?;

    let mut request = client.request(method, url).headers(headers);

    if let Some(mime) = mime {
        request = request.header("Content-Type", mime);
    }

    if let Some(content) = content {
        request = request.body(content);
    }

    request_internal(lua, request, callback).await
}

fn create_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| Error::runtime(format_args!("failed to create http client: {error}")))
}

async fn request_internal(
    lua: Lua,
    request: RequestBuilder,
    callback: Option<Function>,
) -> Result<Value> {
    match request.send().await.and_then(Response::error_for_status) {
        Ok(response) => {
            tracing::info!(
                "http request to {}: status={}",
                response.url(),
                response.status()
            );

            if let Some(callback) = callback {
                let response_table = create_response_table(&lua, response).await?;
                callback
                    .call_async::<()>((Value::Nil, response_table))
                    .await?;
                Ok(Value::Nil)
            } else {
                let content = read_response_body(response).await?;
                Ok(Value::String(lua.create_string(content)?))
            }
        }
        Err(error) => {
            let error_msg = format!("http request failed: {error}");
            tracing::error!("{error_msg}");

            if let Some(callback) = callback {
                callback.call_async::<()>((error_msg, Value::Nil)).await?;
                Ok(Value::Nil)
            } else {
                Err(Error::runtime(error_msg))
            }
        }
    }
}

async fn read_response_body(response: Response) -> Result<String> {
    response
        .text()
        .await
        .map_err(|error| Error::runtime(format_args!("failed to read response body: {error}")))
}

async fn create_response_table(lua: &Lua, response: Response) -> Result<Table> {
    let table = lua.create_table()?;

    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("");
    let status = status.as_u16();
    let mime = response
        .headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let headers = lua.create_table()?;
    for (name, value) in response.headers() {
        headers.set(name.as_str(), value.to_str().unwrap_or(""))?;
    }

    table.set("headers", headers)?;
    table.set("status", status)?;
    table.set("reason", reason)?;
    table.set("mime", mime)?;

    let content = response
        .text()
        .await
        .map_err(|error| Error::runtime(format_args!("failed to read response body: {error}")))?;

    table.set("content", content)?;

    Ok(table)
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;
    module.set("get", lua.create_async_function(get)?)?;
    module.set("post", lua.create_async_function(post)?)?;
    module.set("request", lua.create_async_function(request)?)?;

    libs.set("http", &module)?;
    lua.register_module("http", module)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_request_with_table() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            result_error = nil
            result_response = nil
            
            local req = {
                method = "POST",
                url = "https://httpbin.org/post",
                mime = "application/json",
                headers = { ["X-Custom-Header"] = "test-value" },
                content = '{"key":"value"}'
            }
            
            http.request(req, function(err, resp)
                result_error = err
                result_response = resp
            end)
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result_error: Value = lua.globals().get("result_error").unwrap();
        assert!(result_error.is_nil());

        let result_response: Table = lua.globals().get("result_response").unwrap();
        let status: u16 = result_response.get("status").unwrap();
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_http_request_without_callback() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            
            local req = {
                method = "GET",
                url = "https://httpbin.org/get"
            }
            
            result = http.request(req)
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_http_get_with_callback() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            result_error = nil
            result_response = nil
            
            http.get("https://httpbin.org/get", function(err, resp)
                result_error = err
                result_response = resp
            end)
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result_error: Value = lua.globals().get("result_error").unwrap();
        assert!(result_error.is_nil());

        let result_response: Table = lua.globals().get("result_response").unwrap();
        let status: u16 = result_response.get("status").unwrap();
        assert_eq!(status, 200);

        let content: String = result_response.get("content").unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_http_get_without_callback() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            result = http.get("https://httpbin.org/get")
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_http_post_with_data() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            result_error = nil
            result_response = nil
            
            http.post("https://httpbin.org/post", "test=data", function(err, resp)
                result_error = err
                result_response = resp
            end)
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result_error: Value = lua.globals().get("result_error").unwrap();
        assert!(result_error.is_nil());

        let result_response: Table = lua.globals().get("result_response").unwrap();
        let status: u16 = result_response.get("status").unwrap();
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_http_error_handling() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local http = require("http")
            result_error = nil
            result_response = nil
            
            http.get("https://this-domain-does-not-exist-12345.com", function(err, resp)
                result_error = err
                result_response = resp
            end)
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        let result_error: Value = lua.globals().get("result_error").unwrap();
        assert!(!result_error.is_nil());

        let result_response: Value = lua.globals().get("result_response").unwrap();
        assert!(result_response.is_nil());
    }
}
