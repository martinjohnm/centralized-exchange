

/**
 * Represents a single price level in the order book.
 * Strings are used to maintain high decimal precision.
 */
export interface Level {
    /** Price at this level */
    price: string;
    /** Total volume available at this price */
    quantity: string;
}

export interface DepthUpdate {
    market: number; // Corresponds to MarketId enum
    bids: Level[];
    asks: Level[];
    /** Unix timestamp in microseconds */
    timestamp: number; 
}