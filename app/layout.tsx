import type { Metadata } from "next";
import { SolanaWalletProvider } from "@/components/wallet-provider";
import "./globals.css";
export const metadata: Metadata = { title: "SolanaPredict", description: "A decentralized prediction market built on Solana where users can bet on real-world events with cryptocurrency." };
export default function RootLayout({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en">
            <body><SolanaWalletProvider>{children}</SolanaWalletProvider></body>
        </html>
    );
}