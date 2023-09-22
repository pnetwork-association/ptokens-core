mod check_endpoint;
#[allow(clippy::module_inception)]
mod endpoints;
mod error;

pub use self::{endpoints::Endpoints, error::EndpointError};
