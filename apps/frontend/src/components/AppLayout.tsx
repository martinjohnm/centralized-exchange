import type { ReactNode } from "react"
import { Appbar } from "./Appbar"


export const AppLayout = ({children}: {children: ReactNode}) => {
    return <>
        <Appbar/>
        {children}
    </>
}