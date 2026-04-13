// import { StreamType, type WsIncomingMessage } from "../types/marketTypes";

import { StreamType } from "../types/marketTypes";


const ws_url = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws"

export class SignalingManager {
    private ws : WebSocket;
    private static instance: SignalingManager;
    private bufferedMessages : any[] = [];
    private callbacks: Map<string, { callback: any, id : string }[]>;
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
                const text = await event.data.text()
                const message = JSON.parse(text) ;


                
                const stream = message.stream;

                if (this.callbacks.has(stream)) {
                    this.callbacks.get(stream)?.forEach(({callback} : {callback: any}) => {
                        if (stream === StreamType.CANDLE) {
                            callback(message.data)
                        } 

                        if (stream === StreamType.TRADE) {
                            console.log("hi trade");
                            
                        }

                        if (stream === StreamType.TICKER) {
                            console.log("stream");
                            
                        }
                        if (stream === StreamType.DEPTH) {

                            console.log(message);
                            
                            callback()
                        }
                    })
                }
                
            } catch(e) {
                console.error("Failed to decode binary message:", e);
            }
        }
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

    async registerCallback(type : string, callback: any, id : string) {
        this.callbacks.set(type, (this.callbacks.get(type) || []).concat({callback, id}))
    }

    async deRegisterCallback(type: string, id : string) {
        if (this.callbacks.has(type)) {
            const newCallbacks = this.callbacks.get(type)?.filter(callback => callback.id !== id)?? []
            this.callbacks.set(type, newCallbacks)
        }
    }
}