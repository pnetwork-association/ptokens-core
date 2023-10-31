mod registration_extension;
mod registration_signature;

pub use self::{
    registration_extension::get_registration_extension_tx,
    registration_signature::get_registration_signature,
};
