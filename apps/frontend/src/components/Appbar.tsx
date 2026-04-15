import { useNavigate } from "react-router-dom"


export const Appbar = () => {
    
    const navi = useNavigate()

    return <div className=" border-b border-slate-800 bg-slate-400">
        <div className="flex justify-between items-center p-2">
            <div className="flex">
                <div className={`text-xl pl-4 flex flex-col justify-center cursor-pointer `} onClick={() => {navi("/")}}>
                    Exchange
                </div>
                <div className={`text-sm pt-1 flex flex-col justify-center pl-8 cursor-pointer `} onClick={() => {}}>
                    Markets
                </div>
                {/* <div className={`text-sm pt-1 flex flex-col justify-center pl-8 cursor-pointer `} onClick={() => {navi("/trade")}}>
                    Trade
                </div> */}
            </div>
        </div>
    </div>
}