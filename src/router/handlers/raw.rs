use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
};

use crate::prelude::*;

pub(in crate::router::handlers) fn make_response(
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
                log!(
                    "Overwrote header {overwritten:?} \
                        because another for key {} already existed",
                    header.0
                );
            }
        } else {
            log!("Failed to create header value from {}", header.1);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_header() {
        let headers = [
            (header::ACCEPT, "Not really"),
            (header::ACCEPT, "This again?"),
        ];
        let response = make_response("", 418, &headers);
        assert!(response.headers().get_all(header::ACCEPT).iter().count() == 1);
        assert_eq!(
            response
                .headers()
                .get(header::ACCEPT)
                .unwrap()
                .to_str()
                .unwrap(),
            "This again?",
        );
    }

    #[test]
    fn invalid_header() {
        let response = make_response("", 418, &[(header::MAX_FORWARDS, "\n")]);
        assert!(response.headers().get(header::MAX_FORWARDS).is_none());
    }
}
