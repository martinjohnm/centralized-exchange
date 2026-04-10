import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Landing } from "./page/Landing";
import { Trade } from "./page/Trade";


function App() {

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Landing/>}/>
        <Route path="/trade/:market" element={<Trade/>}/>
      </Routes>
    </BrowserRouter>
  )
}

export default App
