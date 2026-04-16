

import { useState } from "react";
import { MarketId, OrderType, Side, type ExchangeRequest } from "../generated/exchange";
// Import the generated types from your new folder

const BE_URL = import.meta.env.VITE_BACKEND_URL || "http://localhost:3000";

export const usePlaceOrder = () => {
    const [loading, setLoading] = useState(false);

    const generateClientId = (userId: number): bigint => {
        const timestamp = BigInt(Date.now());
        const uid = BigInt(userId);
        // 42 bits for time, 22 bits for user
        return (timestamp & 0x3FFFFFFFFFFn) << 22n | (uid & 0x3FFFFFn);
    };

    const placeOrder = async (uiOrder: {market: MarketId, price: number; quantity: number; symbol: string; side: "BUY" | "SELL" }) => {
        setLoading(true);

        const userId = 123; // In reality, get this from your Auth context
        const clientId = generateClientId(userId);

        // This object now perfectly matches your Rust Protobuf definition
        const request: ExchangeRequest = {
            userId: userId.toString(), // forceLong=string makes this a string
            timestamp: Date.now().toString(),
            create: {
                market : uiOrder.market,
                price: uiOrder.price.toString(),
                quantity: uiOrder.quantity.toString(),
                side: uiOrder.side === "BUY" ? Side.BUY : Side.SELL,
                orderType: OrderType.LIMIT,
                clientId: clientId.toString(), // The 64-bit ID safe as a string
            }
        };

        try {
            const response = await fetch(`${BE_URL}/order/create`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(request),
            });

            return await response.json();
        } catch (error) {
            console.error("Failed to place order:", error);
        } finally {
            setLoading(false);
        }
    };

    return { placeOrder, loading };
};