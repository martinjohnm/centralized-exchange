import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Landing } from "./page/Landing";
import { Trade } from "./page/Trade";
import { AppLayout } from "./components/AppLayout";


function App() {

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<AppLayout><Landing/></AppLayout>}/>
        <Route path="/trade/:market" element={<AppLayout><Trade/></AppLayout>}/>
      </Routes>
    </BrowserRouter>
  )
}

export default App
