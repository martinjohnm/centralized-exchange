import { useEffect, useRef, useState } from "react"

interface Candle {
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  timestamp: number;
}

function App() {

  const WS_URL = import.meta.env.VITE_WS_URL || "ws://127.0.0.1:8080/ws";
  const [newCandle, setNewCandle] = useState<Candle | null>(null);

    // 1. Use a Ref to keep the socket instance across re-renders
  const socketRef = useRef<WebSocket | null>(null);

  const subscribe = (market: string) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      const msg = {
        method : "subscribe",
        params : {
          market
        }
      };

      socketRef.current.send(JSON.stringify(msg));

      
    }
  }
  // // 3. The "Unsubscribe" Helper
  //   const unsubscribe = (market: string) => {
  //     if (socketRef.current?.readyState === WebSocket.OPEN) {
  //       const msg = {
  //         method: "unsubscribe",
  //         params: { market }
  //       };
  //       socketRef.current.send(JSON.stringify(msg));
        

  //     }
  //   };

  useEffect(() => {
    const ws = new WebSocket(WS_URL)
    socketRef.current = ws

    ws.onopen = () => {
      subscribe("btcusdt:5m");
    }

    ws.onmessage = async (event) => {
      const text = await event.data.text()
      const data = JSON.parse(text);

      setNewCandle(data)
    }

    return () => ws.close();

  }, [WS_URL])


  


  // Prevent "Cannot read properties of null" error
  if (!newCandle) {
    return <div>Connecting to Rust Gateway...</div>;
  }

  return (
    <div>
      <h1>Live Market Data</h1>
      <ul>
        <li>Open: {newCandle.open}</li>
        <li>High: {newCandle.high}</li>
        <li>Low: {newCandle.low}</li>
        <li>Close: {newCandle.close}</li>
        <li>Volume: {newCandle.volume}</li>
        <li>Timestamp: {newCandle.timestamp}</li>
      </ul>
      
      <hr />
      
      <h3>Raw JSON String:</h3>
      <pre>{JSON.stringify(newCandle)}</pre>
    </div>
  )
}

export default App
