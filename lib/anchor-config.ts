'use client';
import { Program, AnchorProvider, web3 } from '@coral-xyz/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { useMemo } from 'react';
import idl from './idl.json'; // Make sure you have your IDL file

const programId = new web3.PublicKey(idl.metadata.address);

export const useAnchorProgram = () => {
    const { connection } = useConnection();
    const wallet = useWallet();

    const provider = useMemo(() => {
        if (wallet.publicKey) {
            return new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
        }
        return null;
    }, [connection, wallet]);

    const program = useMemo(() => {
        if (provider) {
            return new Program(idl as any, programId, provider);
        }
        return null;
    }, [provider]);

    return { program, provider };
};