import { useParams } from "react-router-dom";
import { Appbar } from "../components/Appbar";
import { TradeView } from "../components/TradeView";
import { useEffect } from "react";

export const Trade = () => {

  const { market } = useParams<{ market: string }>();

  useEffect(() => {

  }, [market])
  
  if (!market) return <div>{"NO such market available"}</div>

  return <div className="flex flex-row flex-1">
        <div className="flex flex-col flex-1">
            <Appbar/>
            <div className="flex flex-row h-230 border-y border-slate-800">
                <div className="flex flex-col flex-1">
                    <TradeView market={market}/>
                </div>
                <div className="flex flex-col w-62.5 overflow-hidden">
                    {/* <Depth market={market as string} /> */}
                </div>
            </div>
        </div>
        <div className="w-2.5 flex-col border-slate-800 border-l"></div>
        <div>
            <div className="flex flex-col w-62.5">
                {/* <SwapUI market={market as string} /> */}
            </div>
        </div>
    </div>
}