

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

        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);

            console.log(message);
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
}