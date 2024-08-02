use crate::abi;
use antelope::{Asset, ExtendedSymbol, Name};
use substreams::log;
use substreams_antelope::Block;

use std::collections::HashSet;
use substreams_database_change::change::AsString;

pub fn get_balance_delta(block: Block, account: Name, ext_sym: ExtendedSymbol) -> Option<Asset> {
    let sym = ext_sym.get_symbol();
    let contract = ext_sym.get_contract();

    let mut balance_changes: Vec<i64> = Vec::new();
    let mut trx_traces = 0;

    for trx in block.transaction_traces() {
        for db_op in &trx.db_ops {
            // unrwap table operation
            let code = Name::from(db_op.clone().code.as_str());
            let table_name = db_op.clone().table_name;
            let scope = Name::from(db_op.scope.clone().as_str());

            if table_name != "accounts" {
                continue;
            }
            if code != contract {
                continue;
            }
            if account != scope {
                continue;
            }

            trx_traces = trx.action_traces.len(); //prevent non-trade transfers, they have low traces

            let old_balance = abi::parse_balance(db_op.old_data_json.as_str());
            if old_balance.is_none() {
                continue;
            }
            let old_balance = Asset::from(old_balance.unwrap().balance.as_str());
            if old_balance.symbol != sym {
                continue;
            }
            balance_changes.push(old_balance.amount);

            log::debug!("old_balance: {:?}", old_balance);

            let new_balance = abi::parse_balance(db_op.new_data_json.as_str());
            if new_balance.is_none() {
                continue;
            }
            let new_balance = Asset::from(new_balance.unwrap().balance.as_str());
            if new_balance.symbol != sym {
                continue;
            }
            balance_changes.push(new_balance.amount);

            log::debug!("new_balance: {:?}", new_balance);
        }
    }

    if balance_changes.len() == 0 {
        return None;
    }
    if trx_traces <= 10 {
        return None;
    } //prevent non-trade transfers, they have low traces
    let balance_delta = balance_changes.last().unwrap() - balance_changes.first().unwrap();
    log::info!(
        "balance changes: {:?} {:?} {:?} {:?} {:?}",
        balance_delta,
        account.to_string(),
        contract.to_string(),
        sym.code().to_string(),
        balance_changes
    );

    Some(Asset::from_amount(balance_delta, sym))
}

pub fn create_filters(params: &str, key: &str) -> HashSet<String> {
    let mut filter = HashSet::new();
    for part in params.split('&').collect::<Vec<&str>>() {
        let kv = part.split('=').collect::<Vec<&str>>();
        if (kv.len() != 2) || (kv[0] != key) {
            continue;
        }
        for item in kv[1].split(',').collect::<Vec<&str>>() {
            filter.insert(item.as_string());
        }
    }
    log::debug!("filter: {:?} {:?}", key, filter);
    filter
}
