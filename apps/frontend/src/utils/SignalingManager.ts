// import { StreamType, type WsIncomingMessage } from "../types/marketTypes";

import { StreamType, WsOutMessage } from "../generated/exchange";


const ws_url = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws"

export class SignalingManager {
    private ws : WebSocket;
    private static instance: SignalingManager;
    private bufferedMessages : any[] = [];
    private callbacks: Map<StreamType, { callback: any, id : string }[]>;
    private id : number;
    private initialized : boolean = false;

    // singelton pattern
    private constructor() {
        this.ws = new WebSocket(ws_url)
        this.bufferedMessages = [];
        this.callbacks = new Map();
        this.id = 1;
        this.init()
    }

    public static getInstance() {
        if (!this.instance) {
            this.instance = new SignalingManager()
        }
        return this.instance
    }

    init() {
        this.ws.onopen = () => {
            this.initialized = true;
            this.bufferedMessages.forEach(message => {
                this.ws.send(JSON.stringify(message))
            })

            this.bufferedMessages = [];
        }

        this.ws.onmessage = async (event) => {
            
            try {
                // 1. Convert the incoming Blob to a Uint8Array
                const buffer = event.data instanceof Blob 
                    ? await event.data.arrayBuffer() 
                    : event.data;
                const uint8Array = new Uint8Array(buffer);

                // 2. Decode the TOP-LEVEL wrapper (the WsOutMessage)
                const message = WsOutMessage.decode(uint8Array);
                
                // 3. Extract the metadata (stream type and market)
                const streamType = message.stream; // This is now your Enum (0, 1, 2...)

                
                switch (streamType) {
                    case StreamType.CANDLE:
                        this.callbacks.get(streamType)?.forEach(({callback} : {callback: any}) => {
                            callback(message.candle)
                        })
                        break;
                    case StreamType.DEPTH: 
                        this.callbacks.get(streamType)?.forEach(({callback} : {callback: any}) => {
                            callback(message.depth)
                        })
                        break;
                    case StreamType.USER_UPDATES:
                        this.callbacks.get(streamType)?.forEach(({callback} : {callback: any}) => {
                            callback(message.executionReport)
                        })
                        break;
                    default:
                        console.warn("Unknown stream type received:", message.stream);
                }
                
            } catch (e) {
                console.error("Failed to decode unified binary message:", e);
            }
            };
    }

    

    sendMessage(message: any) {
        const messageTosend = {
            ...message,
            id : this.id++
        }

        if (!this.initialized) {
            this.bufferedMessages.push(messageTosend)
            return
        }

        this.ws.send(JSON.stringify(messageTosend))
    }

    async registerCallback(type : StreamType, callback: any, id : string) {
        this.callbacks.set(type, (this.callbacks.get(type) || []).concat({callback, id}))
    }

    async deRegisterCallback(type: StreamType, id : string) {
        if (this.callbacks.has(type)) {
            const newCallbacks = this.callbacks.get(type)?.filter(callback => callback.id !== id)?? []
            this.callbacks.set(type, newCallbacks)
        }
    }
}