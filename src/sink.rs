use std::collections::HashMap;

use substreams::errors::Error;
use substreams::log;
use substreams_antelope::Block;
use antelope::{Name, SymbolCode};
use substreams_sink_prometheus::{PrometheusOperations, Counter, Gauge};
use crate::abi;

#[substreams::handlers::map]
pub fn prom_out(block: Block) -> Result<PrometheusOperations, Error> {

    let mut prom_out = PrometheusOperations::default();

    for trx in block.all_transaction_traces() {
        // Action Traces
        for trace in &trx.action_traces {
            // unwrap action_trace
            let action_trace = trace.action.as_ref().unwrap();
            let name = action_trace.name.clone();
            let account = action_trace.account.clone();

            // label
            let account_label = HashMap::from([("account".to_string(), account.to_string())]);

            // push to prometheus
            if name == "mine" {
                prom_out.push(Counter::from("mine").with(account_label).inc());
			}
        }

        // Database Operations
        for db_op in &trx.db_ops {
            // unrwap table operation
            let contract = db_op.clone().code;
            let table_name = db_op.clone().table_name;
        
            // handle config changes for fast.sx
            if contract == "fast.sx" &&  table_name == "config" {
                // decode table
                let raw_primary_key = Name::from(db_op.primary_key.as_str()).value;            
                let symcode = SymbolCode::from(raw_primary_key).to_string();
                let account = db_op.scope.clone();
                log::info!("contract={} primary_key={} raw_primary_key={} symcode={} account={}", contract, db_op.primary_key, raw_primary_key, symcode, account);
                
                // parse ABIs
                log::debug!("new_data_json={:?}", db_op.new_data_json);
                let config = abi::parse_fast_config(&db_op.new_data_json);
                let max_mine = match config {
                    Some(config) => config.max_mine,
                    None => 0
                };

                // Skip if no balance found
                if max_mine == 0 { continue; }

                // push to prometheus
                prom_out.push(Gauge::from("fast_max_mine").set(max_mine as f64));
            }
        }
    }
    Ok(prom_out)
}
