import { useEffect } from "react";
// import { AskTable } from "./AskTable";
// import { BidTable } from "./BidTable";
import { SignalingManager } from "../../utils/SignalingManager";
import { MarketNames, StreamType } from "../../types/marketTypes";


export const Depth = ({market} : {market: string}) => {

    // const [bids, setBids] = useState<[string, string][]>();
    // const [asks, setAsks] = useState<[string, string][]>();
    // const [price, setPrice] = useState<string>("67");

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

        SignalingManager.getInstance().registerCallback(StreamType.DEPTH, () => {
            // console.log(data);
            
        }, `${marketName}:depth`)


        return () => {
            SignalingManager.getInstance().sendMessage({
                method: "unsubscribe",
                params: { 
                        market: `${marketName}`,
                        stream : `depth` 
                    }
            })
            SignalingManager.getInstance().deRegisterCallback(`${marketName}:depth`, `${marketName}:depth`)
        }
    }, [marketName])


    return <div>
        
        <div>
            
        </div>
        <TableHeader />
        {/* {asks && <AskTable asks={asks} />}
        {price && <div>{price}</div>}
        {bids && <BidTable bids={bids} />} */}
    </div>
}


function TableHeader() {
    return <div className="flex justify-between text-xs">
    <div className="text-white">Price</div>
    <div className="text-slate-500">Size</div>
    <div className="text-slate-500">Total</div>
</div>
}