import { WalletButton } from "@/components/wallet-button";
export default function Home() {
    return (
        <div className="flex flex-col min-h-screen bg-gray-900 text-white">
            <header className="p-4 flex justify-between items-center border-b border-gray-700">
                <h1 className="text-2xl font-bold">SolanaPredict</h1>
                <WalletButton />
            </header>
            <main className="flex-grow flex flex-col items-center justify-center text-center p-8">
                <h2 className="text-4xl font-bold mb-4">Welcome to SolanaPredict</h2>
                <p className="text-lg text-gray-400 max-w-2xl">A decentralized prediction market built on Solana where users can bet on real-world events with cryptocurrency.</p>
                <div className="mt-8 p-6 border border-gray-700 rounded-lg bg-gray-800">
                    <h3 className="text-xl font-semibold">Connect your wallet to get started.</h3>
                    <p className="text-gray-500 mt-2">This is a placeholder UI. The AI will generate the full application based on your prompt.</p>
                </div>
            </main>
        </div>
    );
}