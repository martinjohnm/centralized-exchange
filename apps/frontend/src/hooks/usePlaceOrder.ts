

import { useState } from "react";
import { MarketId, OrderType, Side, type ExchangeRequest } from "../generated/exchange";
// Import the generated types from your new folder

const BE_URL = import.meta.env.VITE_BACKEND_URL || "http://localhost:3000";

export const usePlaceOrder = () => {
    const [loading, setLoading] = useState(false);


    const placeOrder = async (uiOrder: {market: MarketId, price: number; quantity: number;  side: Side }) => {
        setLoading(true);

        const userId = 123; // In reality, get this from your Auth context

        // This object now perfectly matches your Rust Protobuf definition
        const request: ExchangeRequest = {
            userId: userId, // forceLong=string makes this a string
            timestamp: Date.now(),
            create: {
                market : uiOrder.market,
                price: uiOrder.price.toString(),
                quantity: uiOrder.quantity.toString(),
                side: uiOrder.side                                                                                                                                                                                                                                                                                                                                                                                                                                                                              ,
                orderType: OrderType.LIMIT,
                clientId: Date.now(), // The 64-bit ID safe as a string
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