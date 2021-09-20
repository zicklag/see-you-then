<template>
  <q-layout view="lHh Lpr lFf">
    <q-header elevated>
      <q-toolbar>
        <q-toolbar-title> See You Then™ </q-toolbar-title>

        <wallet-connect-button />
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script lang="ts">
import WalletConnectButton from 'components/WalletConnectButton.vue';
import { defineComponent, ref } from 'vue';

export default defineComponent({
  name: 'MainLayout',

  components: { WalletConnectButton },

  setup() {
    return {};
  },
});
</script>
