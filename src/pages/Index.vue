<template>
  <q-page class="row items-center justify-evenly">
    <div class="row">
      <q-date v-model="selectedDateString" landscape class="q-mr-md" />
      <q-list bordered>
        <q-item>
          <q-item-section>
            <q-item-label class="text-h5"> Time Slots</q-item-label>
          </q-item-section>
          <q-item-section avatar>
            <q-btn icon="add" color="primary" round>
              <q-tooltip> Add a new available time slot </q-tooltip>
            </q-btn>
          </q-item-section>
        </q-item>
        <q-separator />
      </q-list>
    </div>
  </q-page>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from 'vue';
import { date } from 'quasar';

export default defineComponent({
  name: 'Index',
  components: {},
  setup() {
    const now = new Date(Date.now());
    const selectedDateString = ref(date.formatDate(now, 'YYYY/MM/DD'));
    const selectedDate = ref(Date.parse(selectedDateString.value));
    watch(selectedDateString, (newSelectedDate) => {
      selectedDate.value = Date.parse(newSelectedDate);
      console.log(date.formatDate(selectedDate.value, 'YYYY/MM/DD'));
    });

    return {
      selectedDateString,
    };
  },
});
</script>
