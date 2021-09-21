<template>
  <q-page class="row items-center justify-evenly">
    <q-dialog v-model="showAddTimeSlotDialog">
      <q-card>
        <q-card-section>
          <div class="text-h6">Create Time Slot</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
            class="q-ma-md"
            v-model="addTimeSlotStartTime"
            filled
            type="time"
            label="Start Time"
          />
          <q-input
            class="q-ma-md"
            v-model="addTimeSlotEndTime"
            filled
            type="time"
            label="End Time"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn
            flat
            label="Add Time Slot"
            icon="add"
            color="primary"
            v-close-popup
            @click="addTimeSlotButton"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
    <div class="row justify-center">
      <q-date
        v-model="selectedDateString"
        landscape
        class="q-ma-md"
        :events="calendarEventsHighlighter"
      />
      <q-list bordered>
        <q-item>
          <q-item-section>
            <q-item-label class="text-h5"> Time Slots</q-item-label>
          </q-item-section>
          <q-item-section avatar>
            <q-btn
              icon="add"
              color="primary"
              round
              @click="showAddTimeSlotDialog = true"
            >
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
import {
  createTimeSlot,
  getTimeSlots,
  subscribeToTimeSlots,
  TimeSlot,
} from '../utils/backend';
import { date } from 'quasar';
import { selectedWallet } from 'src/utils/wallet';

export default defineComponent({
  name: 'Index',
  components: {},
  setup() {
    const publicKey = selectedWallet.value?.adapter.publicKey;
    const now = new Date(Date.now());
    const selectedDateString = ref(date.formatDate(now, 'YYYY/MM/DD'));
    const selectedDate = ref(Date.parse(selectedDateString.value));

    const showAddTimeSlotDialog = ref(false);
    const addTimeSlotStartTime = ref('');
    const addTimeSlotEndTime = ref('');
    const timeSlots = ref<TimeSlot[]>([]);

    // Watch the selected date string and update the selected date
    watch(selectedDateString, (newSelectedDate) => {
      selectedDate.value = Date.parse(newSelectedDate);
      console.log(date.formatDate(selectedDate.value, 'YYYY/MM/DD'));
    });

    if (publicKey) {
      void getTimeSlots(publicKey).then((slots) => {
        timeSlots.value = slots;
      });

      subscribeToTimeSlots(publicKey, (timeSlot) => {
        const newTimeSlots = timeSlots.value.filter((x) => x.id != timeSlot.id);
        newTimeSlots.push(timeSlot);
        timeSlots.value = newTimeSlots;
      });
    }

    return {
      showAddTimeSlotDialog,
      selectedDateString,
      addTimeSlotStartTime,
      addTimeSlotEndTime,
      calendarEventsHighlighter(dateStr: string) {
        const date = new Date(Date.parse(dateStr));

        return timeSlots.value.reduce((acc, slot) => {
          return (
            (slot.time.start.getFullYear() == date.getFullYear() &&
              slot.time.start.getMonth() == date.getMonth() &&
              slot.time.start.getDay() == date.getDay()) ||
            acc
          );
        }, false);
      },
      async addTimeSlotButton() {
        const startDate = new Date(Date.parse(selectedDateString.value));
        startDate.setHours(
          Number.parseInt(addTimeSlotStartTime.value.split(':')[0])
        );
        startDate.setMinutes(
          Number.parseInt(addTimeSlotStartTime.value.split(':')[1])
        );
        const endDate = new Date(Date.parse(selectedDateString.value));
        endDate.setHours(
          Number.parseInt(addTimeSlotEndTime.value.split(':')[0])
        );
        endDate.setMinutes(
          Number.parseInt(addTimeSlotEndTime.value.split(':')[1])
        );

        console.log({
          start: date.formatDate(startDate, 'YYYY/MM/DD HH:mm A'),
          end: date.formatDate(endDate, 'YYYY/MM/DD HH:mm A'),
        });

        await createTimeSlot(startDate, endDate);
      },
    };
  },
});
</script>
