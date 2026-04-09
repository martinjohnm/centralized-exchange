import { useEffect, useRef, useState } from "react"
import { Markets } from "./components/Markets";
import { Appbar } from "./components/Appbar";

interface Candle {
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  timestamp: number;
}

function App() {

// apps/frontend/src/config.ts
const WS_URL = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws";  
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

  console.log(newCandle)
  


  // // Prevent "Cannot read properties of null" error
  // if (!newCandle) {
  //   return <div>Connecting to Rust Gateway...</div>;
  // }

  return (
    <>
      <Appbar/>
      <div className="flex min-h-screen flex-col items-center justify-between p-24 bg-slate-300">
        
        <div>
          <Markets/>
        </div>
      </div>
    </>
  )
}

export default App
