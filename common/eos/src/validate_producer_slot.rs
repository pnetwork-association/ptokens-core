use common::{traits::DatabaseInterface, types::Result};

use crate::{
    eos_block_header::EosBlockHeaderV2,
    eos_constants::PRODUCER_REPS,
    eos_producer_schedule::EosProducerScheduleV2,
    EosState,
};

fn get_producer_index(num_producers: u64, block_timestamp: u64) -> u64 {
    debug!("  num producers: {}", num_producers);
    debug!("block timestamp: {}", block_timestamp);
    (block_timestamp % (num_producers * PRODUCER_REPS)) / PRODUCER_REPS
}

fn validate_producer_slot(schedule: &EosProducerScheduleV2, block: &EosBlockHeaderV2) -> Result<()> {
    let index = get_producer_index(schedule.producers.len() as u64, block.timestamp.as_u32() as u64) as usize;
    if block.producer == schedule.producers[index].producer_name {
        Ok(())
    } else {
        debug!(" calculated index: {index}");
        debug!("expected producer: {}", block.producer.to_string());
        debug!(
            "  actual producer: {}",
            schedule.producers[index].producer_name.to_string()
        );
        Err("producer slot not valid!".into())
    }
}

pub fn validate_producer_slot_of_block_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if cfg!(feature = "non-validating") {
        info!("skipping producer slot validation!");
        Ok(state)
    } else {
        info!("validating slot of producer of block...");
        validate_producer_slot(state.get_active_schedule()?, state.get_eos_block_header()?).and(Ok(state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos_test_utils::get_sample_eos_submission_material_n;

    #[test]
    fn should_validate_producer_slot() {
        let block_header = get_sample_eos_submission_material_n(7).block_header;
        let schedule = EosProducerScheduleV2::from_json(
            "{\"version\":37,\"producers\":[{\"producer_name\":\"alohaeostest\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8JTznQrfvYcoFskidgKeKsmPsx3JBMpTo1jsEG2y1Ho6sQhFuL\",\"weight\":1}]}]},{\"producer_name\":\"atticlabjbpn\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7pfLMz45bKTVqVMfnxktqi6RYjDV46C82Q5eE8NZHM9Nnsai6T\",\"weight\":1}]}]},{\"producer_name\":\"batinthedark\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6dwoM8XGMQn49LokUcLiony7JDkbHrsFDvh5svLvPDkXtvM7oR\",\"weight\":1}]}]},{\"producer_name\":\"bighornsheep\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5xfwWr4UumKm4PqUGnyCrFWYo6j5cLioNGg5yf4GgcTp2WcYxf\",\"weight\":1}]}]},{\"producer_name\":\"bigpolarbear\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6oZi9WjXUcLionUtSiKRa4iwCW5cT6oTzoWZdENXq1p2pq53Nv\",\"weight\":1}]}]},{\"producer_name\":\"clevermonkey\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5mp5wmRyL5RH2JUeEh3eoZxkJ2ZZJ9PVd1BcLioNuq4PRCZYxQ\",\"weight\":1}]}]},{\"producer_name\":\"eosarabianet\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6nrJJGhoZPShQ2T4se2RqxRh5rD2LUvqBK6r5y5VVN9x1oTBwa\",\"weight\":1}]}]},{\"producer_name\":\"eosbarcelona\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8N1MhQpFQR3YABzVp4woPBywQnS5BunJtHv8jxtNQGrGEiTBhD\",\"weight\":1}]}]},{\"producer_name\":\"eosdacserval\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH\",\"weight\":1}]}]},{\"producer_name\":\"eosnationftw\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6Fat9KYfu22yxWJuwjXeWKhCnFxj4GaCQJ7pwjLwpU8XxVzjyi\",\"weight\":1}]}]},{\"producer_name\":\"eosphereiobp\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5P7EBrzje2ZPjYfRNe9aFGvrXiXj2j9xQy3Pj4Jxh3z5P81uGr\",\"weight\":1}]}]},{\"producer_name\":\"funnyhamster\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h\",\"weight\":1}]}]},{\"producer_name\":\"gorillapower\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8X5NCx1Xqa1xgQgBa9s6EK7M1SjGaDreAcLion4kDVLsjhQr9n\",\"weight\":1}]}]},{\"producer_name\":\"hippopotamus\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7qDcxm8YtAZUA3t9kxNGuzpCLioNnzpTRigi5Dwsfnszckobwc\",\"weight\":1}]}]},{\"producer_name\":\"hungryolddog\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6tw3AqqVUsCbchYRmxkPLqGct3vC63cEzKgVzLFcLionoY8YLQ\",\"weight\":1}]}]},{\"producer_name\":\"iliketurtles\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6itYvNZwhqS7cLion3xp3rLJNJAvKKegxeS7guvbBxG1XX5uwz\",\"weight\":1}]}]},{\"producer_name\":\"ivote4eosusa\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8WHzxnaVoXek6mwU7BJiBbyugeqZfb2y2SKa7mVUv8atLfbcjK\",\"weight\":1}]}]},{\"producer_name\":\"junglesweden\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5D1YP3nYVQvE8NPPM5a9wnqVaD54mJAHEuH9vJuNG1E2UsgbY2\",\"weight\":1}]}]},{\"producer_name\":\"lioninjungle\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7ueKyvQJpBLVjuNgLedAgJakw3bLyd4GBx1N4jXswpBhJif5mV\",\"weight\":1}]}]},{\"producer_name\":\"ohtigertiger\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7tigERwXDRuHsok212UDToxFS1joUhAxzvDUhRof8NjuvwtoHX\",\"weight\":1}]}]},{\"producer_name\":\"tokenika4tst\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6wkp1PpqQUgEA6UtgW21Zo3o1XcQeLXzcLLgKcPJhTz2aSF6fz\",\"weight\":1}]}]}]}"
        ).unwrap();
        if let Err(e) = validate_producer_slot(&schedule, &block_header) {
            panic!("Error validating producer slot: {}", e);
        }
    }
}
