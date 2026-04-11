import type { UTCTimestamp } from "lightweight-charts";

export interface RawKline {
    bucket: string;
    open: string;
    high: string;
    low: string;
    close: string;
    volume: string;
}

// The Mapper function
export const mapKlineToChart = (raw: RawKline): any => {
    return {
        // Convert ISO String "2026-04-08T..." to Unix Timestamp (seconds)
        time: (Math.floor(new Date(raw.bucket).getTime() / 1000)) as UTCTimestamp,
        open: parseFloat(raw.open),
        high: parseFloat(raw.high),
        low: parseFloat(raw.low),
        close: parseFloat(raw.close),
    };
};