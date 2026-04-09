

export const Appbar = () => {
    

    return <div className=" border-b border-slate-800 bg-slate-400">
        <div className="flex justify-between items-center p-2">
            <div className="flex">
                <div className={`text-xl pl-4 flex flex-col justify-center cursor-pointer `} onClick={() => {}}>
                    Exchange
                </div>
                <div className={`text-sm pt-1 flex flex-col justify-center pl-8 cursor-pointer `} onClick={() => {}}>
                    Markets
                </div>
                <div className={`text-sm pt-1 flex flex-col justify-center pl-8 cursor-pointer `} onClick={() => {}}>
                    Trade
                </div>
            </div>
            <div className="flex">
                <div className="px-2 mr-2">
                    <button className="bg-green-500 p-2">Deposit</button>
                    <button className="bg-red-500 p-2">Withdraw</button>
                </div>
            </div>
        </div>
    </div>
}