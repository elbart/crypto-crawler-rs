use crate::WSClient;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, time::Duration};

use super::{
    utils::CHANNEL_PAIR_DELIMITER,
    ws_client_internal::{MiscMessage, WSClientInternal},
    Candlestick, Level3OrderBook, OrderBook, OrderBookTopK, Ticker, Trade, BBO,
};
use log::*;
use serde_json::Value;

pub(super) const EXCHANGE_NAME: &str = "bitmex";

const WEBSOCKET_URL: &str = "wss://www.bitmex.com/realtime";

const CLIENT_PING_INTERVAL_AND_MSG: (u64, &str) = (5, "ping");

// Too many args sent. Max length is 20
const MAX_CHANNELS_PER_COMMAND: usize = 20;

/// The WebSocket client for BitMEX.
///
/// BitMEX has Swap and Future markets.
///
///   * WebSocket API doc: <https://www.bitmex.com/app/wsAPI>
///   * Trading at: <https://www.bitmex.com/app/trade/>
pub struct BitmexWSClient {
    client: WSClientInternal,
}

fn channels_to_commands(channels: &[String], subscribe: bool) -> Vec<String> {
    let raw_channels: Vec<&String> = channels.iter().filter(|ch| !ch.starts_with('{')).collect();
    let mut all_commands: Vec<String> = channels
        .iter()
        .filter(|ch| ch.starts_with('{'))
        .map(|s| s.to_string())
        .collect();

    if !raw_channels.is_empty() {
        let n = raw_channels.len();
        for i in (0..n).step_by(MAX_CHANNELS_PER_COMMAND) {
            let chunk: Vec<&String> =
                (&raw_channels[i..(std::cmp::min(i + MAX_CHANNELS_PER_COMMAND, n))]).to_vec();
            all_commands.append(&mut vec![format!(
                r#"{{"op":"{}","args":{}}}"#,
                if subscribe {
                    "subscribe"
                } else {
                    "unsubscribe"
                },
                serde_json::to_string(&chunk).unwrap()
            )])
        }
    };

    all_commands
}

// see https://www.bitmex.com/app/wsAPI#Response-Format
fn on_misc_msg(msg: &str) -> MiscMessage {
    if msg == "pong" {
        return MiscMessage::Pong;
    }
    let resp = serde_json::from_str::<HashMap<String, Value>>(msg);
    if resp.is_err() {
        error!("{} is not a JSON string, {}", msg, EXCHANGE_NAME);
        return MiscMessage::Misc;
    }
    let obj = resp.unwrap();

    if obj.contains_key("error") {
        let error_msg = obj.get("error").unwrap().as_str().unwrap();
        let code = obj.get("status").unwrap().as_i64().unwrap();

        match code {
            // Rate limit exceeded
            429 => {
                error!("Received {} from {}", msg, EXCHANGE_NAME);
                std::thread::sleep(Duration::from_secs(3));
            }
            400 => {
                if error_msg.starts_with("Unknown") {
                    panic!("Received {} from {}", msg, EXCHANGE_NAME);
                } else if error_msg.starts_with("You are already subscribed to this topic") {
                    info!("Received {} from {}", msg, EXCHANGE_NAME)
                } else {
                    warn!("Received {} from {}", msg, EXCHANGE_NAME);
                }
            }
            _ => error!("Received {} from {}", msg, EXCHANGE_NAME),
        }
        MiscMessage::Misc
    } else if obj.contains_key("success") || obj.contains_key("info") {
        info!("Received {} from {}", msg, EXCHANGE_NAME);
        MiscMessage::Misc
    } else if obj.contains_key("table") && obj.contains_key("action") && obj.contains_key("data") {
        MiscMessage::Normal
    } else {
        warn!("Received {} from {}", msg, EXCHANGE_NAME);
        MiscMessage::Misc
    }
}

fn to_raw_channel(channel: &str, pair: &str) -> String {
    format!("{}{}{}", channel, CHANNEL_PAIR_DELIMITER, pair)
}

#[rustfmt::skip]
impl_trait!(Trade, BitmexWSClient, subscribe_trade, "trade", to_raw_channel);
#[rustfmt::skip]
impl_trait!(BBO, BitmexWSClient, subscribe_bbo, "quote", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBook, BitmexWSClient, subscribe_orderbook, "orderBookL2_25", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBookTopK, BitmexWSClient, subscribe_orderbook_topk, "orderBook10", to_raw_channel);

impl Ticker for BitmexWSClient {
    fn subscribe_ticker(&self, _pairs: &[String]) {
        panic!("BitMEX WebSocket does NOT have ticker channel");
    }
}

fn to_candlestick_raw_channel(pair: &str, interval: usize) -> String {
    let interval_str = match interval {
        60 => "1m",
        300 => "5m",
        3600 => "1h",
        86400 => "1d",
        _ => panic!("BitMEX has intervals 1m,5m,1h,1d"),
    };
    format!("tradeBin{}:{}", interval_str, pair)
}

impl_candlestick!(BitmexWSClient);

panic_l3_orderbook!(BitmexWSClient);

impl_new_constructor!(
    BitmexWSClient,
    EXCHANGE_NAME,
    WEBSOCKET_URL,
    channels_to_commands,
    on_misc_msg,
    Some(CLIENT_PING_INTERVAL_AND_MSG),
    None
);
impl_ws_client_trait!(BitmexWSClient);

#[cfg(test)]
mod tests {
    #[test]
    fn test_one_channel() {
        let commands = super::channels_to_commands(&vec!["trade:XBTUSD".to_string()], true);
        assert_eq!(1, commands.len());
        assert_eq!(r#"{"op":"subscribe","args":["trade:XBTUSD"]}"#, commands[0]);
    }

    #[test]
    fn test_multiple_channels() {
        let commands = super::channels_to_commands(
            &vec![
                "trade:XBTUSD".to_string(),
                "quote:XBTUSD".to_string(),
                "orderBookL2_25:XBTUSD".to_string(),
                "tradeBin1m:XBTUSD".to_string(),
            ],
            true,
        );
        assert_eq!(1, commands.len());
        assert_eq!(
            r#"{"op":"subscribe","args":["trade:XBTUSD","quote:XBTUSD","orderBookL2_25:XBTUSD","tradeBin1m:XBTUSD"]}"#,
            commands[0]
        );
    }
}
