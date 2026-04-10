import { useParams } from "react-router-dom";
import { Appbar } from "../components/Appbar"


export const Trade = () => {

    const { market } = useParams();
    return <>
              <Appbar/>
              <div>
                {market}
              </div>
              <div className="flex min-h-screen flex-col items-center justify-between p-24 bg-slate-300">
                
                <div>
                  
                </div>
              </div>
    </>
}