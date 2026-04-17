import { useEffect, useState } from "react";
import { AskTable } from "./AskTable";
import { BidTable } from "./BidTable";
import { SignalingManager } from "../../utils/SignalingManager";
import { MarketNames } from "../../types/marketTypes";
import { DepthUpdate, Level, StreamType } from "../../generated/exchange";


export const Depth = ({market} : {market: string}) => {

    const [bids, setBids] = useState<Level[]>();
    const [asks, setAsks] = useState<Level[]>();
    const price = 65000;
    const marketName = MarketNames[Number(market)];
    // for the depth update
    useEffect(() => {

        SignalingManager.getInstance().sendMessage({
            method : "subscribe",
            params: {
                market: `${marketName}`,
                stream: "depth"
            }
        })

        SignalingManager.getInstance().registerCallback(StreamType.DEPTH, (data: DepthUpdate) => {
            setAsks(data.asks);
            setBids(data.bids);
            // setPrice(data.bids[0].price)
            
        }, `${marketName}:depth`)


        return () => {
            SignalingManager.getInstance().sendMessage({
                method: "unsubscribe",
                params: { 
                        market: `${marketName}`,
                        stream : `depth` 
                    }
            })
            SignalingManager.getInstance().deRegisterCallback(StreamType.DEPTH, `${marketName}:depth`)
        }
    }, [marketName])


    return <div>
        
        <div>
            
        </div>
        <TableHeader />
        {asks && <AskTable asks={asks} />}
        {price && <div>{price}</div>}
        {bids && <BidTable bids={bids} />}
    </div>
}


function TableHeader() {
    return <div className="flex justify-between text-xs">
    <div className="text-white">Price</div>
    <div className="text-slate-500">Size</div>
    <div className="text-slate-500">Total</div>
</div>
}