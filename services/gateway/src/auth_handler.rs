use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, Uri},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use crate::AppState;

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let target_url = format!("{}{}", state.user_management_url, path_query);

    let (parts, body) = req.into_parts();
    let method = parts.method;
    let headers = parts.headers;

    // Read body
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => return Response::builder().status(400).body(Body::from(e.to_string())).unwrap(),
    };

    // Build reqwest request
    let mut request_builder = state.http_client.request(method.clone(), target_url.clone());
    request_builder = request_builder.headers(headers);
    request_builder = request_builder.body(reqwest::Body::from(body_bytes));

    let request = match request_builder.build() {
        Ok(req) => req,
        Err(e) => return Response::builder().status(500).body(Body::from(e.to_string())).unwrap(),
    };

    match state.http_client.execute(request).await {
        Ok(res) => {
            let mut response = Response::builder().status(res.status());
            
            for (key, value) in res.headers() {
                // Skip hop-by-hop headers that might cause conflicts with Axum/Hyper's management
                if key != "transfer-encoding" && key != "connection" && key != "content-length" && key != "date" {
                    response = response.header(key, value);
                }
            }
            
            let bytes = match res.bytes().await {
                Ok(b) => b,
                Err(e) => return Response::builder().status(502).body(Body::from(e.to_string())).unwrap(),
            };
            
            response.body(Body::from(bytes)).unwrap()
        }
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            Response::builder()
                .status(502)
                .body(Body::from(format!("Bad Gateway: {}", e)))
                .unwrap()
        }
    }
}
