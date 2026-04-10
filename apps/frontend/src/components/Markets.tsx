import { useNavigate } from "react-router-dom";
import { MarketId } from "../proto/exchange";
import { MarketNames } from "../types/marketTypes";

export const Markets = () => {


  return (
    <div className="flex flex-col flex-1 max-w-[1280px] w-full">
      <div className="flex flex-col min-w-[700px] flex-1 w-full">
        <div className="flex flex-col w-full rounded-lg bg-baseBackgroundL1 px-5 py-3">
          <table className="w-full table-auto">
            <MarketHeader />
            <MarketComponent name={MarketNames[MarketId.BTC_USDT]} market_id={MarketId.BTC_USDT} price={2345} day_change={34} day_volume={2345} market_cap={4590}/>
            <MarketComponent name={MarketNames[MarketId.ETH_USDT]} market_id={MarketId.ETH_USDT} price={1232} day_change={65} day_volume={5674} market_cap={8970}/>
          </table>
        </div>
      </div>
    </div>
  );
};

function MarketHeader() {
  return (
      <thead className="bg-gray-400">
        <tr className="">
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              Name<span className="w-[16px]"></span>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              Price<span className="w-[16px]"></span>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              Market Cap<span className="w-[16px]"></span>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              24h Volume
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="lucide lucide-arrow-down h-4 w-4"
              >
                <path d="M12 5v14"></path>
                <path d="m19 12-7 7-7-7"></path>
              </svg>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              24h Change<span className="w-[16px]"></span>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
                Trade<span className="w-[16px]"></span>
            </div>
          </th>
        </tr>
      </thead>
  );
}

interface MarketParams {
  name : string;
  market_id : number;
  price : number;
  market_cap : number;
  day_volume: number;
  day_change : number
}
function MarketComponent(params : MarketParams) {
  const navi = useNavigate()
  return <thead>
        <tr className="">
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              {params.name}
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              {params.price}
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              {params.market_cap}
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              <p>{params.day_volume}</p>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="lucide lucide-arrow-down h-4 w-4"
              >
                <path d="M12 5v14"></path>
                <path d="m19 12-7 7-7-7"></path>
              </svg>
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none">
              {params.day_change}
            </div>
          </th>
          <th className="px-2 py-3 text-left text-sm font-normal text-baseTextMedEmphasis">
            <div className="flex items-center gap-1 cursor-pointer select-none hover:underline" onClick={() => {navi(`/trade/${params.market_id}`)}}>
                    Trade
                </div>
          </th>
        </tr>
      </thead>
}