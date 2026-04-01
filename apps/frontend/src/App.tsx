import { useEffect, useState } from "react"

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

  useEffect(() => {
    const socket = new WebSocket(WS_URL)

    socket.onmessage = async (event) => {
      try {
        const text = await event.data.text();
        const data = JSON.parse(text);
        setNewCandle(data);
      } catch (err) {
        console.error("Error:", err);
      }
    };

    return () => socket.close();

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
