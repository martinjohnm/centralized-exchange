import axios from "axios"
import type { RawKline } from "../types/KlineTypes";
import type { CandlestickData, UTCTimestamp } from "lightweight-charts";
import type { MarketId } from "../generated/exchange";

const BE_URL = import.meta.env.VITE_BACKEND_URL || "http://localhost:3000"


export async function getKlines(symbol : MarketId, interval: string, limit: number): Promise<CandlestickData[]> {
    const response = await axios.get<RawKline[]>(
        `${BE_URL}/get-klines?symbol=${symbol}&interval=${interval}&limit=${limit}`
    );
    return response.data.map((raw) => {
        let timestampInSeconds: number;

        // Check if bucket is a numeric string (Microseconds) or ISO String
        if (!isNaN(Number(raw.bucket))) {
            // Logic for Microsecond Epoch String -> Seconds
            // Use BigInt to prevent precision loss before division
            timestampInSeconds = Number(BigInt(raw.bucket) / BigInt(1000000));
        } else {
            // Logic for ISO String -> Seconds
            timestampInSeconds = Math.floor(new Date(raw.bucket).getTime() / 1000);
        }

        return {
            time: timestampInSeconds as UTCTimestamp,
            open: parseFloat(raw.open),
            high: parseFloat(raw.high),
            low: parseFloat(raw.low),
            close: parseFloat(raw.close),
        };
    });
    
}