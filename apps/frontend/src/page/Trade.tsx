import { useParams } from "react-router-dom";
import { Appbar } from "../components/Appbar"
import { useEffect } from "react";
import { SignalingManager } from "../utils/SignalingManager";


export const Trade = () => {

    const { market } = useParams();

    useEffect(() => {
      const init = async() => {
          SignalingManager.getInstance().sendMessage(
            {
              "method": "subscribe",
              "params": {
                  "market": "btcusdt:5m"
              }
            })
          
          return () => {
            SignalingManager.getInstance().sendMessage(
              {
                "method": "unsubscribe",
                "params": {
                    "market": "btcusdt:5m"
                }
              }
            )
          }
      }

      init()
    })
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