import { MarketId } from "../proto/exchange";



// Map IDs to Names for Display
export const MarketNames: Record<number, string> = {
    [MarketId.BTC_USDT]: "btcusdt",
    [MarketId.ETH_USDT]: "ethusdt",
};

export const CandleTime: Record<number, string> = {
    5 : "5m",
    15: "15m",
    1: "1m"
}


export const StreamType = {
    CANDLE: "candle",
    TRADE: "trade",
    TICKER: "ticker",
    DEPTH : "depth"
} as const;


export interface CandleData {
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    timestamp: number;
}


export interface TradeData {
    price: string;
    quantity: string;
    side: 'buy' | 'sell';
}


export interface TickerData {
    price: string;
    quantity: string;
    side: 'buy' | 'sell';
}

// This matches Rust Level struct perfectly for the UI
interface Level {
    price: string;    // "65000.50"
    quantity: string; // "1.25000000"
}

// The full Depth object coming through the WebSocket
export interface DepthData {
    market: number;   // matching your u64 market ID
    bids: Level[];    // Sorted High -> Low
    asks: Level[];    // Sorted Low -> High
    timestamp: number;
}


// 3. Create the Unified Message Type
export type WsIncomingMessage = 
    | { stream: typeof StreamType.CANDLE; data: CandleData }
    | { stream: typeof StreamType.TRADE; data: TradeData }
    | { stream: typeof StreamType.TICKER, data: TickerData}
    | { stream: typeof StreamType.DEPTH, data: DepthData };