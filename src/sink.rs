use std::collections::HashMap;

use substreams::errors::Error;
use substreams::log;
use substreams_antelope::Block;
use antelope::{Name, SymbolCode, Asset, Symbol};
use substreams_sink_prometheus::{PrometheusOperations, Counter, Gauge};
use crate::abi;

#[substreams::handlers::map]
pub fn prom_out(block: Block) -> Result<PrometheusOperations, Error> {

    let mut prom_out = PrometheusOperations::default();
    let producer = block.clone().header.unwrap().producer.to_string();
    let producer_label = HashMap::from([("producer".to_string(), producer)]);

    for trx in block.all_transaction_traces() {
        // Action Traces
        for trace in &trx.action_traces {
            // unwrap action_trace
            let action_trace = trace.action.as_ref().unwrap();
            let name = action_trace.name.clone();
            let account = action_trace.account.clone();

            // handle token transfers
            match abi::parse_transfer(&action_trace.json_data) {
                Some(transfer) => {
                    let from = transfer.from;
                    let to = transfer.to;
                    let amount = Asset::from(transfer.quantity.as_str()).value();
                    let symbol = Symbol::from(transfer.quantity.as_str()).code().to_string();
                    let symbol_label = HashMap::from([("symbol".to_string(), symbol.to_string())]);

                    // handle token transfers
                    if from == "trader.sx" && to == "fee.sx" {
                        prom_out.push(Counter::from("trade_profit_total").with(symbol_label.clone()).add(amount));
                    }
                    if from == "cpu.sx" && to == "eosio.rex" {
                        prom_out.push(Counter::from("powerup_total").with(symbol_label.clone()).add(amount));
                    }
                    if from == "fee.sx" && to == "cpu.sx" {
                        prom_out.push(Counter::from("fee_cpu_total").with(symbol_label.clone()).add(amount));
                    }
                    if from == "fee.sx" && to == "ops.sx" {
                        prom_out.push(Counter::from("fee_ops_total").with(symbol_label.clone()).add(amount));
                    }
                    if from == "fee.sx" && to == "push.sx" {
                        prom_out.push(Counter::from("fee_push_total").with(symbol_label.clone()).add(amount));
                    }
                },
                None => {}
            }

            // skip additional receivers (i.e. not the contract account)
            if trace.receiver != account { continue; }

            // push to prometheus
            if name == "mine" && account == "push.sx" {
                let mine = abi::parse_mine(&action_trace.json_data);
                let executor = match mine {
                    Some(mine) => mine.executor,
                    None => { continue; }
                };
                let executor_label = HashMap::from([("executor".to_string(), executor.to_string())]);
                prom_out.push(Counter::from("mine").inc());
                prom_out.push(Counter::from("mine_by_producer").with(producer_label.clone()).inc());
                prom_out.push(Counter::from("mine_by_executor").with(executor_label).inc());
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
