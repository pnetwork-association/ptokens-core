mod debug_functions;
mod debug_signatories;
mod debug_signatory;
mod debug_signatures;
mod eip_712_signature_hash_generator;
mod get_debug_signature_info;
#[cfg(test)]
mod test_utils;
mod validate_debug_command_signature;

#[cfg(not(feature = "no-safe-debug-signers"))]
pub use self::debug_signatories::SAFE_DEBUG_SIGNATORIES;
pub use self::{
    debug_functions::{debug_add_debug_signer, debug_add_multiple_debug_signers, debug_remove_debug_signer},
    debug_signatories::{DebugSignatories, DEBUG_SIGNATORIES_DB_KEY},
    debug_signatory::DebugSignatory,
    get_debug_signature_info::get_debug_signature_info,
    validate_debug_command_signature::validate_debug_command_signature,
};

#[macro_use]
extern crate common;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
