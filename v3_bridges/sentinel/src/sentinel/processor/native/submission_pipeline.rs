use std::result::Result;

use common_eth::{EthSubmissionMaterial, EthSubmissionMaterials};
use lib::SentinelError;

fn process_native_material_single(material: &EthSubmissionMaterial) -> Result<(), SentinelError> {
    // TODO Real pipeline
    let n = material.get_block_number()?;
    info!("Finished processing native block {n}!");
    Ok(())
}

fn process_native_material(batch: &EthSubmissionMaterials) -> Result<Vec<()>, SentinelError> {
    info!("Processing native submission material...");
    let r = batch
        .iter()
        .map(process_native_material_single)
        .collect::<Result<Vec<()>, SentinelError>>();
    info!("Finished processing native submission material!");
    r
}

// TODO need a oneshot channel for a synce to throw stuff to this thread.
// Which otherwise will do nothing until messages are received.
// all the native side needs to do is parse events that we're looking for and _save_ them. That's
// basically it! Need to save them in some performant DB, along with a "seen on opposite chain"
// type flag too.
//
// also need some way to initialize the chain since we'll need some chain concept in order to have
// the concept of confirmations
//
// also need to figure out how we're going to manage the database stuff - use something in memory
// that we can still use with references, then some sort of channel stuff to pass messages in
// between.
//
// NEED to figure out the db stuff pretty quickly to be honest, because that's the hard bit I'd
// say.
//
// also need a broadcaster, but that should be a simple module right? Which can just stay in a
// quiet loop, watching a db for txs that it might have to sign.
