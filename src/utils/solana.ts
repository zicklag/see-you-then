import { Connection } from '@solana/web3.js';

export const solanaUrl = 'https://api.devnet.solana.com';
// export const solanaUrl = 'http://localhost:8899';

export const connection = new Connection(solanaUrl, 'confirmed');
