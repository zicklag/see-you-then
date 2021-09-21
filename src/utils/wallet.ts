import { WalletAdapter } from '@solana/wallet-adapter-base';
import { ref } from 'vue';
import { Wallet } from '../lib/wallets/types';

export interface SelectedWallet {
  wallet: Wallet;
  adapter: WalletAdapter;
}

export const selectedWallet = ref<null | SelectedWallet>(null);
