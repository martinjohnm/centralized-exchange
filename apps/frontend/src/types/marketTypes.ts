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

// 3. Create the Unified Message Type
export type WsIncomingMessage = 
    | { stream: typeof StreamType.CANDLE; data: CandleData }
    | { stream: typeof StreamType.TRADE; data: TradeData }
    | { stream: typeof StreamType.TICKER, data: TickerData};