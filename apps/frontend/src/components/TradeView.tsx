import { CandleTime, MarketNames } from "../types/marketTypes";
import { useEffect, useRef, useState } from "react";
import { getKlines } from "../utils/httpClient";
import { SignalingManager } from "../utils/SignalingManager";
import type { CandlestickData } from "lightweight-charts";
import { ChartManager } from "../utils/ChartManager";
import { Candle, StreamType } from "../generated/exchange";


const INITIAL_CANDLE_COUNT = 200;


export const TradeView = ({market} : {market: string}) => {

    
    const marketName = MarketNames[Number(market)];
    const [candleTime, setCandleTime] = useState<string>(CandleTime[5]);

    const chartManagerRef = useRef<ChartManager>(null);
    const [chartManageer, setChartManageer] = useState<ChartManager>()

    if (chartManageer) {
        console.log("")
    }
    const chartRef = useRef<HTMLDivElement>(null);
// 
//     
    useEffect(() => {
        // 1. Define the async call
        const init = async () => {

            // 1. PHYSICAL CLEANUP: Wipe the div completely
            if (chartRef.current) {
                chartRef.current.innerHTML = ""; 
            }
            let klines: CandlestickData[] = [];
            
            try {
                klines = await getKlines(Number(market), candleTime, INITIAL_CANDLE_COUNT);
                
            } catch (e) {
                console.log(e);
            }

            if (chartRef.current) {

                if (chartManagerRef.current) {
                    chartManagerRef.current.destroy();
                }

                const chartManager = new ChartManager (
                    chartRef.current,
                    klines,
                    {
                        background: "#0e0f14",
                        color: "white",
                    }
                )

                setChartManageer(chartManager)

                SignalingManager.getInstance().registerCallback(StreamType.CANDLE, (data: Candle) => {
                    

                    // 1. Convert Microseconds to Seconds (Number)
                    // data.timestamp / 1,000,000
                    const rawSeconds = Math.floor(Number(data.timestamp) / 1000000);

                    // 2. Map Timeframe String to Seconds
                    // "1m" -> 60, "5m" -> 300, etc.
                    const intervalMinutes = parseInt(candleTime); 
                    const intervalSeconds = intervalMinutes * 60;
                    
                    // 3. Round down to the Bucket
                    const bucketTimestamp = Math.floor(rawSeconds / intervalSeconds) * intervalSeconds;
                    
                    chartManager.update({
                        // 1. Rename 'timestamp' to 'time' (and ensure it's in seconds)
                        timestamp: bucketTimestamp,
                        
                        // 2. Map your price fields
                        open: parseFloat(data.open),
                        high: parseFloat(data.high),
                        low: parseFloat(data.low),
                        
                        // 3. Rename 'current' to 'close'
                        current: parseFloat(data.close) 
                    });
                
                }, `${StreamType.CANDLE}-${marketName}:${candleTime}`)

                SignalingManager.getInstance().sendMessage({
                    method: "subscribe",
                    params: { 
                        market: `${marketName}`,
                        stream : `candles_${candleTime}` 
                    }
                });
            }

            
        };

        init();

        // 2. Return the cleanup function DIRECTLY to useEffect
        return () => {
            SignalingManager.getInstance().sendMessage({
                method: "unsubscribe",
                params: { 
                        market: `${marketName}`,
                        stream : `candles_${candleTime}` 
                    }
            });

            SignalingManager.getInstance().deRegisterCallback(StreamType.CANDLE, `${StreamType.CANDLE}-${marketName}:${candleTime}`)

            if (chartManagerRef.current) {
                chartManagerRef.current.destroy();
                chartManagerRef.current = null;
            }
        };
        
    }, [candleTime, marketName, market]); // Empty dependency array ensures this runs on mount/unmount
  return <>
            <div className="flex items-center px-4 py-2 border-b border-gray-800 gap-4">
              <span className="font-bold text-lg">{marketName}</span>
              <TimeSelector selected={candleTime} onSelect={setCandleTime} />
            </div>
            <div ref={chartRef} className="" style={{ height: "100%", width: "100%", marginTop: 4 }}></div>
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