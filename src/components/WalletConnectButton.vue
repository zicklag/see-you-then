<template>
  <div>
    <!-- The Wallet Select Dialog -->
    <q-dialog v-model="showDialog">
      <q-card>
        <q-list>
          <q-item>
            <q-item-section>
              <q-item-label> Select Wallet </q-item-label>
            </q-item-section>
          </q-item>
          <q-item
            clickable
            v-ripple
            v-for="wallet in wallets"
            :key="wallet.name"
            @click="connectWallet(wallet)"
          >
            <q-item-section avatar>
              <q-avatar>
                <q-img :src="wallet.icon" />
              </q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label>
                {{ wallet.name }}
              </q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </q-dialog>

    <!-- The Button -->
    <q-btn
      v-if="!selectedWallet"
      label="Connect Wallet"
      :flat="flat"
      @click="showDialog = true"
      :color="color"
    />
    <q-btn
      v-else-if="selectedWallet && !selectedWallet.adapter.connected"
      :flat="flat"
      label="Connecting..."
      :color="color"
    />
    <q-btn
      v-else
      :flat="flat"
      @click="selectedWallet?.adapter.disconnect()"
      :color="color"
    >
      <q-tooltip>Logout</q-tooltip>
      <div class="row items-center">
        <q-avatar class="q-mr-sm">
          <q-img :src="selectedWallet.wallet.icon" />
        </q-avatar>
        <div>
          {{ selectedWallet.wallet.name }} Connected (
          <span style="text-transform: none">
            {{ getShortPubKeyString() }}
          </span>
          )
        </div>
      </div>
    </q-btn>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, triggerRef } from 'vue';
import { selectedWallet } from '../utils/wallet';
import {
  getSolletWallet,
  getPhantomWallet,
  getSolletExtensionWallet,
  getSolflareWallet,
  getSolflareWebWallet,
  Wallet,
} from '../lib/wallets';
import { useQuasar } from 'quasar';

export default defineComponent({
  name: 'WalletConnectButton',
  props: {
    color: String,
    flat: Boolean,
  },
  setup() {
    const $q = useQuasar();
    const showDialog = ref(false);
    const wallets = [
      getPhantomWallet(),
      getSolletWallet(),
      getSolletExtensionWallet(),
      getSolflareWallet(),
      getSolflareWebWallet(),
    ] as Wallet[];

    return {
      showDialog,
      wallets,
      selectedWallet,
      async connectWallet(wallet: Wallet) {
        showDialog.value = false;
        const adapter = wallet.adapter();

        adapter.on('connect', () => {
          triggerRef(selectedWallet);
          if (selectedWallet.value) {
            $q.notify({
              type: 'positive',
              message: `Successfully connected to ${selectedWallet.value.wallet.name} wallet`,
            });
          }
        });
        adapter.on('ready', () => {
          triggerRef(selectedWallet);
        });
        adapter.on('disconnect', () => {
          if (selectedWallet.value) {
            $q.notify({
              type: 'positive',
              message: `Disconnected from ${selectedWallet.value.wallet.name} wallet`,
            });
            selectedWallet.value = null;
          }
        });
        adapter.on('error', (e) => {
          $q.notify({
            type: 'negative',
            message: 'Wallet error' + (e.message != '' ? ': ' + e.message : ''),
          });

          if (!selectedWallet.value?.adapter.connected) {
            selectedWallet.value = null;
          }
        });

        selectedWallet.value = {
          adapter,
          wallet,
        };

        await adapter.connect();

        triggerRef(selectedWallet);
      },
      getShortPubKeyString(): string {
        const pubkey = selectedWallet.value?.adapter.publicKey?.toString();

        if (pubkey) {
          return (
            pubkey.substr(0, 4) +
            '...' +
            pubkey.substring(pubkey.length - 4, pubkey.length)
          );
        } else {
          return '';
        }
      },
    };
  },
});
</script>
