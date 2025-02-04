use crate::WSClient;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

use super::super::ws_client_internal::{MiscMessage, WSClientInternal};
use super::super::{Candlestick, Level3OrderBook, OrderBook, OrderBookTopK, Ticker, Trade, BBO};
use crate::clients::utils::ensure_frame_size;

use log::*;
use serde_json::Value;

const EXCHANGE_NAME: &str = "bitget";

const WEBSOCKET_URL: &str = "wss://csocketapi.bitget.com/ws/v1";

// https://bitgetlimited.github.io/apidoc/en/swap/#brief-introduction
// System will auto-disconnect while subscription has been done within 30sec
// or no ping command sent by user after 30sec after ws is connected
const CLIENT_PING_INTERVAL_AND_MSG: (u64, &str) = (30, "ping");

// User has the opinion to subscribe 1 or more channels, total length of multiple channel can not exceeds 4096 bytes.
const WS_FRAME_SIZE: usize = 4096;

/// The WebSocket client for Bitget swap markets.
///
/// * WebSocket API doc: <https://bitgetlimited.github.io/apidoc/en/swap/#websocketapi>
/// * Trading at: <https://www.bitget.com/en/swap/>
pub struct BitgetSwapWSClient {
    client: WSClientInternal,
}

fn topics_to_command(chunk: &[String], subscribe: bool) -> String {
    format!(
        r#"{{"op":"{}","args":{}}}"#,
        if subscribe {
            "subscribe"
        } else {
            "unsubscribe"
        },
        serde_json::to_string(chunk).unwrap()
    )
}

fn channels_to_commands(channels: &[String], subscribe: bool) -> Vec<String> {
    ensure_frame_size(channels, subscribe, topics_to_command, WS_FRAME_SIZE, None)
}

fn on_misc_msg(msg: &str) -> MiscMessage {
    if msg == "pong" {
        return MiscMessage::Pong;
    }
    let obj = serde_json::from_str::<HashMap<String, Value>>(msg).unwrap();

    if obj.contains_key("event") {
        let event = obj.get("event").unwrap().as_str().unwrap();
        if event == "error" {
            error!("Received {} from {}", msg, EXCHANGE_NAME);
            panic!("Received {} from {}", msg, EXCHANGE_NAME);
        } else {
            info!("Received {} from {}", msg, EXCHANGE_NAME);
            MiscMessage::Misc
        }
    } else if obj.contains_key("table") && obj.contains_key("data") {
        if let Some(arr) = obj.get("data").unwrap().as_array() {
            if arr.is_empty() {
                info!("data field is empty {} from {}", msg, EXCHANGE_NAME);
                MiscMessage::Misc
            } else {
                MiscMessage::Normal
            }
        } else {
            MiscMessage::Normal
        }
    } else if obj.contains_key("action") {
        let action = obj.get("action").unwrap().as_str().unwrap();
        if action == "ping" {
            info!("Received {} from {}", msg, EXCHANGE_NAME);
        } else {
            warn!("Received {} from {}", msg, EXCHANGE_NAME);
        }
        MiscMessage::Misc
    } else {
        warn!("Received {} from {}", msg, EXCHANGE_NAME);
        MiscMessage::Misc
    }
}

fn to_raw_channel(channel: &str, symbol: &str) -> String {
    format!("swap/{}:{}", channel, symbol)
}

#[rustfmt::skip]
impl_trait!(Trade, BitgetSwapWSClient, subscribe_trade, "trade", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBookTopK, BitgetSwapWSClient, subscribe_orderbook_topk, "depth5", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBook, BitgetSwapWSClient, subscribe_orderbook, "depth", to_raw_channel);
#[rustfmt::skip]
impl_trait!(Ticker, BitgetSwapWSClient, subscribe_ticker, "ticker", to_raw_channel);

impl BBO for BitgetSwapWSClient {
    fn subscribe_bbo(&self, _symbols: &[String]) {
        panic!("Bitget does NOT have BBO channel");
    }
}

fn to_candlestick_raw_channel(symbol: &str, interval: usize) -> String {
    let valid_set: Vec<usize> = vec![60, 300, 900, 1800, 3600, 14400, 43200, 86400, 604800];
    if !valid_set.contains(&interval) {
        let joined = valid_set
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");
        panic!("Bitget available intervals: {}", joined);
    }
    let channel = format!("candle{}s", interval);
    to_raw_channel(&channel, symbol)
}

impl_candlestick!(BitgetSwapWSClient);

panic_l3_orderbook!(BitgetSwapWSClient);

impl_new_constructor!(
    BitgetSwapWSClient,
    EXCHANGE_NAME,
    WEBSOCKET_URL,
    channels_to_commands,
    on_misc_msg,
    Some(CLIENT_PING_INTERVAL_AND_MSG),
    None
);
impl_ws_client_trait!(BitgetSwapWSClient);
