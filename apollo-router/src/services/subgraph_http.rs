#![allow(missing_docs)] // FIXME

use http::StatusCode;
use serde_json_bytes::ByteString;
use serde_json_bytes::Map as JsonMap;
use serde_json_bytes::Value;
use static_assertions::assert_impl_all;
use tower::BoxError;

use crate::error::Error;
use crate::graphql;
use crate::json_ext::Object;
use crate::json_ext::Path;
use crate::Context;

pub type BoxService = tower::util::BoxService<Request, Response, BoxError>;
pub type BoxCloneService = tower::util::BoxCloneService<Request, Response, BoxError>;
pub type ServiceResult = Result<Response, BoxError>;
pub type Body = hyper::Body;

assert_impl_all!(Request: Send);
#[non_exhaustive]
pub struct Request {
    pub subgraph_request: http::Request<Body>,
    pub context: Context,
}

#[buildstructor::buildstructor]
impl Request {
    /// This is the constructor (or builder) to use when constructing a real Request.
    ///
    /// Required parameters are required in non-testing code to create a Request.
    #[builder(visibility = "pub")]
    fn new(subgraph_request: http::Request<Body>, context: Context) -> Request {
        Self {
            subgraph_request,
            context,
        }
    }

    /// This is the constructor (or builder) to use when constructing a "fake" Request.
    ///
    /// This does not enforce the provision of the data that is required for a fully functional
    /// Request. It's usually enough for testing, when a fully consructed Request is
    /// difficult to construct and not required for the pusposes of the test.
    #[builder(visibility = "pub")]
    fn fake_new(
        subgraph_request: Option<http::Request<Body>>,
        context: Option<Context>,
    ) -> Request {
        Request::new(
            subgraph_request.unwrap_or_default(),
            context.unwrap_or_default(),
        )
    }
}

assert_impl_all!(Response: Send);
#[derive(Debug)]
#[non_exhaustive]
pub struct Response {
    pub response: http::Response<Body>,

    pub context: Context,
}

#[buildstructor::buildstructor]
impl Response {
    /// This is the constructor (or builder) to use when constructing a real Response.
    ///
    /// The parameters are not optional, because in a live situation all of these properties must be
    /// set and be correct to create a Response.
    #[builder(visibility = "pub")]
    fn new(
        label: Option<String>,
        data: Option<Value>,
        path: Option<Path>,
        errors: Vec<Error>,
        extensions: Object,
        status_code: Option<StatusCode>,
        context: Context,
    ) -> Response {
        // Build a response
        let res = graphql::Response::builder()
            .and_label(label)
            .data(data.unwrap_or_default())
            .and_path(path)
            .errors(errors)
            .extensions(extensions)
            .build();

        // Build an http Response
        let response = http::Response::builder()
            .status(status_code.unwrap_or(StatusCode::OK))
            .body(hyper::Body::from(
                serde_json::to_vec(&res).expect("Response is serializable; qed"),
            ))
            .expect("building a response cannot fail; qed");

        Self { response, context }
    }

    /// This is the constructor (or builder) to use when constructing a "fake" Response.
    ///
    /// This does not enforce the provision of the data that is required for a fully functional
    /// Response. It's usually enough for testing, when a fully consructed Response is
    /// difficult to construct and not required for the pusposes of the test.
    #[builder(visibility = "pub")]
    fn fake_new(
        label: Option<String>,
        data: Option<Value>,
        path: Option<Path>,
        errors: Vec<Error>,
        // Skip the `Object` type alias in order to use buildstructor’s map special-casing
        extensions: JsonMap<ByteString, Value>,
        status_code: Option<StatusCode>,
        context: Option<Context>,
    ) -> Response {
        Response::new(
            label,
            data,
            path,
            errors,
            extensions,
            status_code,
            context.unwrap_or_default(),
        )
    }

    /// This is the constructor (or builder) to use when constructing a Response that represents a global error.
    /// It has no path and no response data.
    /// This is useful for things such as authentication errors.
    #[builder(visibility = "pub")]
    fn error_new(
        errors: Vec<Error>,
        status_code: Option<StatusCode>,
        context: Context,
    ) -> Result<Response, BoxError> {
        Ok(Response::new(
            Default::default(),
            Default::default(),
            Default::default(),
            errors,
            Default::default(),
            status_code,
            context,
        ))
    }
}