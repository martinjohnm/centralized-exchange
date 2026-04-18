import type { Level } from "../../types/depthTypes";

export const AskTable = ({ asks }: { asks: Level[] }) => {
    
    const relevantAsks = asks.slice(0, 15)
    const maxTotal = relevantAsks.reduce((acc, level) => acc + Number(level.quantity), 0);

    const asksWithTotal: [string, string, number][] = relevantAsks.map((level, index) => {
        // We calculate the sum of all quantities from index 0 up to the current index
        const totalSoFar = relevantAsks
            .slice(0, index + 1)
            .reduce((sum, item) => sum + Number(item.quantity), 0);

        return [level.price, level.quantity, totalSoFar];
    });

    asksWithTotal.reverse();

    return <div>
        {asksWithTotal.map(([price, quantity, total]) => <Ask maxTotal={maxTotal} key={price} price={price} quantity={quantity} total={total} />)}
    </div>
}
function Ask({price, quantity, total, maxTotal}: {price: string, quantity: string, total: number, maxTotal: number}) {
    return <div
    style={{
        display: "flex",
        position: "relative",
        width: "100%",
        backgroundColor: "transparent",
        overflow: "hidden",
    }}
>
    <div
        style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: `${(100 * total) / maxTotal}%`,
        height: "100%",
        background: "rgba(228, 75, 68, 0.325)",
        transition: "width 0.3s ease-in-out",
        }}
    ></div>
    <div className="flex justify-between text-xs w-full">
        <div>
            {price}
        </div>
        <div>
            {quantity}
        </div>
        <div>
            {total?.toFixed(2)}
        </div>
    </div>
    </div>
}