import axios from "axios"

const BE_URL = import.meta.env.VITE_BACKEND_URL || "http://localhost:3000"


export async function getKlines() {
    const response = await axios.get(`${BE_URL}/get-klines?symbol=1_3&interval=5m`)
    const data = response.data;
    console.log(data);
    
}