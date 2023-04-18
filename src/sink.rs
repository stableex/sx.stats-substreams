use std::collections::HashMap;

use substreams::errors::Error;
use substreams::log;
use substreams_antelope::Block;
use antelope::{Name, SymbolCode, Asset};
use substreams_sink_prometheus::{PrometheusOperations, Counter, Gauge};
use crate::abi;

#[substreams::handlers::map]
pub fn prom_out(block: Block) -> Result<PrometheusOperations, Error> {

    let mut prom_out = PrometheusOperations::default();
    let producer = block.clone().header.unwrap().producer.to_string();
    let producer_label = HashMap::from([("producer".to_string(), producer)]);
    
    //let mut jamestaggartbalance: f64 = 0.0000;
	
    let mut jamestaggartoldeos: f64 = 0.0000;
    let mut jamestaggartneweos: f64 = 0.0000;
    let mut jamestaggartoldusdt: f64 = 0.0000;
    let mut jamestaggartnewusdt: f64 = 0.0000;

    for trx in block.all_transaction_traces() {
        // Action Traces
        for trace in &trx.action_traces {
            // unwrap action_trace
            let action_trace = trace.action.as_ref().unwrap();
            let name = action_trace.name.clone();
            let account = action_trace.account.clone();
			

            // skip additional receivers (i.e. not the contract account)
            if trace.receiver != account { continue; }

            // handle token transfers
            match abi::parse_transfer(&action_trace.json_data) {
                Some(transfer) => {
                    let from = transfer.from;
                    let to = transfer.to;
                    let memo = transfer.memo;
                    let quantity = Asset::from(transfer.quantity.as_str());
                    let amount = quantity.value();
                    let symbol = quantity.symbol.code().to_string();
                    let transfer_label = HashMap::from([
                        ("symbol".to_string(), symbol.to_string()),
                        ("from".to_string(), from.to_string()),
                        ("to".to_string(), to.to_string()),
                        ("memo".to_string(), memo.to_string()),
                    ]);

                    // profit from trader.sx
                    if from == "trader.sx" && to == "fee.sx" {
                        let symbol_label = HashMap::from([
                            ("owner".to_string(), "trader.sx".to_string()),
                            ("symbol".to_string(), symbol.to_string())
                        ]);
                        prom_out.push(Counter::from("profit").with(symbol_label.clone()).add(amount));
                    }

                    // ignore accounts
                    if to == "trader.sx" || from == "trader.sx" { continue; }
                    if to == "curve.sx" || from == "curve.sx" { continue; }

                    // include sx suffixes accounts (i.e. cpu.sx, ops.sx, push.sx)
                    if Name::from(from.as_str()).suffix() == Name::from("sx") {
                        prom_out.push(Counter::from("transfers").with(transfer_label.clone()).add(amount));
                    }

                },
                None => {}
            }

            // EXTERNAL fundfordream
            if name == "logs" && account == "hezdshrynage" {
                match abi::parse_fundfordream(&action_trace.json_data) {
                    Some(transfer) => {
                        let m = transfer.m;
                        let parts: Vec<&str> = m.split('|').collect();
                        if parts.len() > 2 { continue; }
                        let profit = Asset::from(parts[0]);
                        // let balance = Asset::from(parts[1]);
                        let symbol_label = HashMap::from([
                            ("owner".to_string(), "fundfordream".to_string()),
                            ("symbol".to_string(), profit.symbol.code().to_string())
                        ]);
                        prom_out.push(Counter::from("profit").with(symbol_label.clone()).add(profit.value()));
                    },
                    None => {}
                }
            }
			
            // push mines
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
            let account = db_op.scope.clone();
			
            // EXTERNAL jamestaggart get first and laast balance (EOS, USDT)
            if contract == "noloss111111" {
					
	    match abi::parse_jamestaggart(&db_op.clone().old_data_json) {
		Some(json) => {
		    let balance = json.balance;
		    let balances = Asset::from(balance.as_str());
		    let symbol = balances.symbol.code().to_string();
		    let value = balances.value();
			
                    if symbol == "EOS" {
	                if jamestaggartoldeos == 0.0000 {
		            jamestaggartoldeos = value;
	                }
                    }

                    if symbol == "USDT" {
	                if jamestaggartoldusdt == 0.0000 {
		            jamestaggartoldusdt = value;
	                }
                    }
                },
                None => {}
            }
	
            match abi::parse_jamestaggart(&db_op.clone().new_data_json) {
                Some(json) => {
                    let balance = json.balance;
                    let balances = Asset::from(balance.as_str());
                    let symbol = balances.symbol.code().to_string();
                    let value = balances.value();

                    if symbol == "EOS" {
                        jamestaggartneweos = value;

                        if jamestaggartoldeos == 0.0000 {
	                    jamestaggartoldeos = value;
                        }
                    }
                    else if symbol == "USDT" {
                        jamestaggartnewusdt = value;

                        if jamestaggartoldusdt == 0.0000 {
	                    jamestaggartoldusdt = value;
                        }
                    }
                },
                None => {}
            }
            }

            // handle config changes for fast.sx
            if contract == "fast.sx" && table_name == "config" {
                // decode table
                let raw_primary_key = Name::from(db_op.primary_key.as_str()).value;
                let symcode = SymbolCode::from(raw_primary_key).to_string();
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

    // EXTERNAL jamestaggart finalize delta (EOS, USDT)
    if (jamestaggartneweos > jamestaggartoldeos) && jamestaggartoldeos != 0.0000 {
        let symbol_label = HashMap::from([
	    ("owner".to_string(), "jamestaggart".to_string()),
	    ("symbol".to_string(), "EOS".to_string())
        ]);
        
        //Floating-Point Arithmetic issue (4 decimal points)
        let profit: f64 = ((jamestaggartneweos - jamestaggartoldeos) * 10000.0).round() / 10000.0;
        prom_out.push(Counter::from("profit").with(symbol_label.clone()).add(profit));
    }

    if (jamestaggartnewusdt > jamestaggartoldusdt) && jamestaggartoldusdt != 0.0000{
        let symbol_label = HashMap::from([
	    ("owner".to_string(), "jamestaggart".to_string()),
	    ("symbol".to_string(), "USDT".to_string())
        ]);

        //Floating-Point Arithmetic issue (4 decimal points)
        let profit: f64 = ((jamestaggartnewusdt - jamestaggartoldusdt) * 10000.0).round() / 10000.0;

        prom_out.push(Counter::from("profit").with(symbol_label.clone()).add(profit));
    } 
		
    Ok(prom_out)
}

