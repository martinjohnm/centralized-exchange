import { useParams } from "react-router-dom";
import { Appbar } from "../components/Appbar"
import { useEffect } from "react";
import { SignalingManager } from "../utils/SignalingManager";


export const Trade = () => {

    const { market } = useParams<{ market: string }>();
  useEffect(() => {
    // 1. Define the async call
    const startSubscription = async () => {

        SignalingManager.getInstance().registerCallback("btcusdt:5m", (data: any) => {
          console.log(data);
          
        }, `BTCUDT:5M`)

        SignalingManager.getInstance().sendMessage({
          method: "subscribe",
          params: { market: "btcusdt:5m" }
        });
    };

    startSubscription();

    // 2. Return the cleanup function DIRECTLY to useEffect
    return () => {
      SignalingManager.getInstance().sendMessage({
        method: "unsubscribe",
        params: { market: "btcusdt:5m" }
      });
    };
  }, []); // Empty dependency array ensures this runs on mount/unmount
    return <>
              <Appbar/>
              <div>
                {market}
              </div>
              <div className="flex min-h-screen flex-col items-center justify-between p-24 bg-slate-300">
                
                <div>
                  
                </div>
              </div>
    </>
}