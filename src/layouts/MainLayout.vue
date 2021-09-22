<template>
  <q-layout view="lHh Lpr lFf">
    <q-header elevated>
      <q-toolbar>
        <q-toolbar-title>
          <router-link
            :to="{ name: 'index' }"
            style="color: white; text-decoration: none"
          >
            See You Then™
          </router-link>
        </q-toolbar-title>

        <wallet-connect-button flat />
      </q-toolbar>
    </q-header>

    <q-page-container>
      <transition name="fade">
        <q-card
          v-if="!selectedWallet?.adapter.connected"
          class="absolute-center"
        >
          <q-card-section class="text-body1">
            Connect a wallet to
            {{
              route.name == 'schedule'
                ? 'schedule a meeting'
                : 'access your calendar'
            }}.
          </q-card-section>
          <q-card-section class="row justify-center">
            <wallet-connect-button color="primary" />
          </q-card-section>
        </q-card>
        <router-view v-else />
      </transition>
    </q-page-container>
  </q-layout>
</template>

<style lang="scss">
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

<script lang="ts">
import WalletConnectButton from 'components/WalletConnectButton.vue';
import { defineComponent } from 'vue';
import { selectedWallet } from '../utils/wallet';
import { useRoute } from 'vue-router';

export default defineComponent({
  name: 'MainLayout',

  components: { WalletConnectButton },

  setup() {
    const route = useRoute();

    return {
      selectedWallet,
      route,
    };
  },
});
</script>
