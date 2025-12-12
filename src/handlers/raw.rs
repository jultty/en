use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
};

pub fn make_response(
    body: &str,
    status_code: u16,
    headers: &[(header::HeaderName, &str)],
) -> Response<Body> {
    let mut response = Response::new(Body::from(body.to_owned()));

    *response.status_mut() = StatusCode::from_u16(status_code)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    for header in headers {
        if let Ok(wrapped) = HeaderValue::from_str(header.1) {
            if let Some(overwritten) =
                response.headers_mut().insert(header.0.clone(), wrapped)
            {
                eprintln!(
                    "[make_response] Overwrote header {overwritten:?} \
                        because another for key {} already existed",
                    header.0
                );
            }
        } else {
            eprintln!(
                "[make_response] Failed to wrap header value {}",
                header.1
            );
        }
    }

    response
}
