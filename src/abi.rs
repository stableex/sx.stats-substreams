use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Balance {
    pub balance: String,
}

pub fn parse_balance(data_json: &str) -> Option<Balance> {
    match serde_json::from_str(data_json) {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FastConfig {
    pub status: String,
    pub trade_amount: u64,
    pub min_profit: u64,
    pub max_mine: u64,
}

pub fn parse_fast_config(data_json: &str) -> Option<FastConfig> {
    match serde_json::from_str(data_json) {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Mine {
    pub executor: String,
    pub nonce: u64,
}

pub fn parse_mine(data_json: &str) -> Option<Mine> {
    match serde_json::from_str(data_json) {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub quantity: String,
    pub memo: String,
}

pub fn parse_transfer(data_json: &str) -> Option<Transaction> {
    match serde_json::from_str(data_json) {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FundforDreamLog {
    pub m: String
}

pub fn parse_fundfordream(data_json: &str) -> Option<FundforDreamLog> {
    match serde_json::from_str(data_json) {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}
