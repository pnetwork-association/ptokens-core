mod check_endpoint;
mod endpoints_struct;
mod error;

pub use self::{check_endpoint::check_endpoint, endpoints_struct::Endpoints, error::EndpointError};
