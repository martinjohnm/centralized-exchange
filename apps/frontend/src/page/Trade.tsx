import { useParams } from "react-router-dom";
import { Appbar } from "../components/Appbar"
import { useEffect, useState } from "react";
import { SignalingManager } from "../utils/SignalingManager";
import { CandleTime, MarketNames, StreamType } from "../types/marketTypes";


export const Trade = () => {

  const { market } = useParams<{ market: string }>();
  const marketName = MarketNames[Number(market)];
  const [candleTime, setCandleTime] = useState<string>(CandleTime[5]);
  
  useEffect(() => {
    // 1. Define the async call
    const startSubscription = async () => {

        SignalingManager.getInstance().registerCallback(StreamType.CANDLE, (data: any) => {
          console.log(data);
          
        }, `${StreamType.CANDLE}-${marketName}:${candleTime}`)

        SignalingManager.getInstance().sendMessage({
          method: "subscribe",
          params: { market: `${marketName}:${candleTime}` }
        });
    };

    startSubscription();

    // 2. Return the cleanup function DIRECTLY to useEffect
    return () => {
      SignalingManager.getInstance().sendMessage({
        method: "unsubscribe",
        params: { market: `${marketName}:${candleTime}` }
      });

      SignalingManager.getInstance().deRegisterCallback(StreamType.CANDLE, `${StreamType.CANDLE}-${marketName}:${candleTime}`)
    };
  }, [candleTime, marketName]); // Empty dependency array ensures this runs on mount/unmount
  return <>
            <Appbar/>
            <div className="flex items-center px-4 py-2 border-b border-gray-800 gap-4">
              <span className="font-bold text-lg">{marketName}</span>
              <TimeSelector selected={candleTime} onSelect={setCandleTime} />
            </div>
            <div className="flex min-h-screen flex-col items-center justify-between p-24 bg-slate-300">
              
              <div>
                
              </div>
            </div>
  </>
}
const TimeSelector = ({ selected, onSelect }: { selected: string; onSelect: (t: string) => void }) => {
  const options = ["1m", "5m", "15m"];
  return (
    <div className="flex bg-[#1e2329] p-1 rounded">
      {options.map((tf) => (
        <button
          key={tf}
          onClick={() => onSelect(tf)}
          className={`px-3 py-1 text-xs font-medium rounded transition-all ${
            selected === tf 
              ? "bg-[#2b3139] text-[#f0b90b]" 
              : "text-gray-400 hover:text-gray-200"
          }`}
        >
          {tf}
        </button>
      ))}
    </div>
  );
};