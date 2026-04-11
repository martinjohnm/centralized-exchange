import axios from "axios"

const BE_URL = import.meta.env.VITE_BACKEND_URL || "http://localhost:3000"


export async function getKlines(symbol : string, interval: string, limit: number) {
    const response = await axios.get(`${BE_URL}/get-klines?symbol=${symbol}&interval=${interval}&limit=${limit}`)
    const data = response.data;
    console.log(data);
    
}