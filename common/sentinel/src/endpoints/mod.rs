mod check_endpoint;
#[allow(clippy::module_inception)]
mod endpoints;
mod error;

pub use self::{check_endpoint::check_endpoint, endpoints::Endpoints, error::EndpointError};
