import { CandlestickSeries, ColorType, createChart, CrosshairMode, type ISeriesApi, type UTCTimestamp } from "lightweight-charts";



export class ChartManager {
    private candleSeries : ISeriesApi<"Candlestick">;
    // private lastUpdateTime: number = 0;
    private chart: any;
    // private currentBar : {
    //     open: number | null,
    //     high: number | null,
    //     low: number | null,
    //     close: number | null
    // } = {
    //     open : null,
    //     high: null,
    //     low : null,
    //     close: null
    // }

    constructor (
        ref: any,
        initialData: any[],
        layout: { background: string, color: string }
    ) {

        const chart = createChart(ref, {
            autoSize: true,
            overlayPriceScales: {
                ticksVisible: true,
                borderVisible: true
            }, 
            crosshair: {
                mode: CrosshairMode.Normal
            },
            rightPriceScale: {
                visible: true,
                ticksVisible: true,
                entireTextOnly: true
            },

            grid: {
                horzLines: {
                    visible: false
                }, 
                vertLines: {
                    visible: false
                }
            },
            layout : {
                background: {
                    type: ColorType.Solid,
                    color: layout.background
                },
                textColor: "white"
            }
        })

        this.chart = chart;
        this.candleSeries = chart.addSeries(CandlestickSeries);

        this.candleSeries.setData(
            initialData.map((data) => ({
                time: data.time as UTCTimestamp, // Use 'time', not 'timestamp'
                open: Number(data.open),
                high: Number(data.high),
                low: Number(data.low),
                close: Number(data.close),
            }))
        );
        console.log("hai this is data from cahrt",initialData);
    }

    public update(updatedcandle: {
            low: number,
            high: number,
            open: number,
            current: number,
            timestamp : number
    }) {
        
        this.candleSeries.update({
            time: updatedcandle.timestamp as UTCTimestamp,
            close : updatedcandle.current,
            low : updatedcandle.low,
            high : updatedcandle.high,
            open : updatedcandle.open,
        })

        // this.lastUpdateTime = updatedcandle.timestamp
    }

    public destroy() {
        this.chart.remove();
    }
}