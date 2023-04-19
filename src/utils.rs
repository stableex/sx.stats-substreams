use antelope::{Name, Asset, ExtendedSymbol};
use substreams_antelope::Block;
use substreams::log;
use crate::abi;

pub fn get_balance_delta(block: Block, account: Name, ext_sym: ExtendedSymbol) -> Option<Asset> {
    let sym = ext_sym.get_symbol();
    let contract = ext_sym.get_contract();

    let mut balance_changes: Vec<i64> = Vec::new();
    for trx in block.all_transaction_traces() {
        for db_op in &trx.db_ops {
            // unrwap table operation
            let code = Name::from(db_op.clone().code.as_str());
            let table_name = db_op.clone().table_name;
            let scope = Name::from(db_op.scope.clone().as_str());

            if table_name != "accounts" { continue; }
            if code != contract { continue; }
            if account != scope { continue; }
            let balance = abi::parse_balance(db_op.new_data_json.as_str());
            if balance.is_none() { continue; }
            let balance = Asset::from(balance.unwrap().balance.as_str());
            if balance.symbol != sym { continue; }
            balance_changes.push(balance.amount);
        }
    }
    if balance_changes.len() == 0 { return None; }
    let balance_delta = balance_changes.last().unwrap() - balance_changes.first().unwrap();
    log::info!("balance changes: {:?} {:?} {:?} {:?} {:?}", balance_delta, account.to_string(), contract.to_string(), sym.code().to_string(), balance_changes);
    Some(Asset::from_amount(balance_delta, sym))
}