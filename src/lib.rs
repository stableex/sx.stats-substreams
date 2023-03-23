use std::collections::HashMap;

use substreams::errors::Error;
use substreams::log;
use substreams_antelope::Block;
use substreams_sink_prometheus::{PrometheusOperations, Counter, Gauge};

#[substreams::handlers::map]
pub fn prom_out(block: Block) -> Result<PrometheusOperations, Error> {

    let mut prom_out = PrometheusOperations::default();

    for trx in block.all_transaction_traces() {
        for trace in &trx.action_traces {
            let action_trace = trace.action.as_ref().unwrap();
            let account_label = HashMap::from([("account".to_string(), action_trace.account.to_string())]);

            if action_trace.name == "mine" {
                prom_out.push(Counter::from("mine").with(account_label).inc());
			}
        }
    }
    Ok(prom_out)
}
