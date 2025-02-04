use crate::WSClient;
use std::sync::mpsc::Sender;

use super::super::ws_client_internal::WSClientInternal;
use super::super::{Candlestick, Level3OrderBook, OrderBook, OrderBookTopK, Ticker, Trade, BBO};
use super::utils::{
    channels_to_commands, on_misc_msg, to_candlestick_raw_channel_shared, to_raw_channel,
    CLIENT_PING_INTERVAL_AND_MSG, EXCHANGE_NAME,
};

const INVERSE_SWAP_WEBSOCKET_URL: &str = "wss://fx-ws.gateio.ws/v4/ws/btc";
const LINEAR_SWAP_WEBSOCKET_URL: &str = "wss://fx-ws.gateio.ws/v4/ws/usdt";

/// The WebSocket client for Gate InverseSwap market.
///
/// * WebSocket API doc: <https://www.gate.io/docs/futures/ws/en/index.html>
/// * Trading at <https://www.gate.io/cn/futures_trade/BTC/BTC_USD>
pub struct GateInverseSwapWSClient {
    client: WSClientInternal,
}

/// The WebSocket client for Gate LinearSwap market.
///
/// * WebSocket API doc: <https://www.gate.io/docs/futures/ws/en/index.html>
/// * Trading at <https://www.gate.io/cn/futures_trade/USDT/BTC_USDT>
pub struct GateLinearSwapWSClient {
    client: WSClientInternal,
}

#[rustfmt::skip]
impl_trait!(Trade, GateInverseSwapWSClient, subscribe_trade, "futures.trades", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBook, GateInverseSwapWSClient, subscribe_orderbook, "futures.order_book_update", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBookTopK, GateInverseSwapWSClient, subscribe_orderbook_topk, "futures.order_book", to_raw_channel);
#[rustfmt::skip]
impl_trait!(BBO, GateInverseSwapWSClient, subscribe_bbo, "futures.book_ticker", to_raw_channel);
#[rustfmt::skip]
impl_trait!(Ticker, GateInverseSwapWSClient, subscribe_ticker, "futures.tickers", to_raw_channel);

#[rustfmt::skip]
impl_trait!(Trade, GateLinearSwapWSClient, subscribe_trade, "futures.trades", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBook, GateLinearSwapWSClient, subscribe_orderbook, "futures.order_book_update", to_raw_channel);
#[rustfmt::skip]
impl_trait!(OrderBookTopK, GateLinearSwapWSClient, subscribe_orderbook_topk, "futures.order_book", to_raw_channel);
#[rustfmt::skip]
impl_trait!(BBO, GateLinearSwapWSClient, subscribe_bbo, "futures.book_ticker", to_raw_channel);
#[rustfmt::skip]
impl_trait!(Ticker, GateLinearSwapWSClient, subscribe_ticker, "futures.tickers", to_raw_channel);

fn to_candlestick_raw_channel(pair: &str, interval: usize) -> String {
    to_candlestick_raw_channel_shared("futures", pair, interval)
}

impl_candlestick!(GateInverseSwapWSClient);
impl_candlestick!(GateLinearSwapWSClient);

panic_l3_orderbook!(GateInverseSwapWSClient);
panic_l3_orderbook!(GateLinearSwapWSClient);

impl_new_constructor!(
    GateInverseSwapWSClient,
    EXCHANGE_NAME,
    INVERSE_SWAP_WEBSOCKET_URL,
    channels_to_commands,
    on_misc_msg,
    Some(CLIENT_PING_INTERVAL_AND_MSG),
    None
);
impl_ws_client_trait!(GateInverseSwapWSClient);

impl_new_constructor!(
    GateLinearSwapWSClient,
    EXCHANGE_NAME,
    LINEAR_SWAP_WEBSOCKET_URL,
    channels_to_commands,
    on_misc_msg,
    Some(CLIENT_PING_INTERVAL_AND_MSG),
    None
);
impl_ws_client_trait!(GateLinearSwapWSClient);
