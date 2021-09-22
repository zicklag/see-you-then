<template>
  <q-page class="column justify-center items-center">
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
            stack-label
          />
          <q-input
            class="q-ma-md"
            v-model="addTimeSlotEndTime"
            filled
            type="time"
            label="End Time"
            stack-label
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
    <q-card class="text-center q-pa-md q-ma-md" v-if="$route.name == 'index'">
      Share this
      <router-link
        :to="{
          name: 'schedule',
          params: { id: calendarPubkey ? calendarPubkey.toString() : '' },
        }"
      >
        link</router-link
      >
      to allow people to reserve spots spots on your calendar.
    </q-card>
    <q-card v-else class="text-center q-pa-md q-ma-md">
      Schedule a meeting with
      {{ calendarPubkey ? calendarPubkey.toString() : '' }}
    </q-card>
    <div class="row justify-center">
      <q-date
        v-model="selectedDateString"
        landscape
        class="q-ma-md"
        :events="calendarEventsHighlighter"
      />
      <q-list bordered separator>
        <q-item>
          <q-item-section>
            <q-item-label class="text-h5"> Time Slots</q-item-label>
          </q-item-section>
          <q-item-section avatar>
            <q-btn
              v-if="$route.name == 'index'"
              icon="add"
              color="primary"
              round
              @click="showAddTimeSlotDialog = true"
            >
              <q-tooltip> Add a new available time slot </q-tooltip>
            </q-btn>
          </q-item-section>
        </q-item>
        <q-item
          v-for="timeSlot in currentDayTimeSlots"
          :key="timeSlot.id"
          :clickable="$route.name == 'schedule'"
          @click="scheduleMeetingButton(timeSlot)"
        >
          <q-item-section>
            <q-item-label>
              {{ format_time(timeSlot.time.start) }} -
              {{ format_time(timeSlot.time.end) }}
            </q-item-label>
            <q-item-label caption>
              {{
                timeSlot.scheduledWith
                  ? `Scheduled with ${format_pubkey(timeSlot.scheduledWith)}`
                  : 'Open'
              }}
            </q-item-label>
          </q-item-section>
        </q-item>
        <q-item v-if="currentDayTimeSlots.length == 0">
          <q-item-section>
            <q-item-label caption> No time slots for this day </q-item-label>
          </q-item-section>
        </q-item>
        <q-separator />
      </q-list>
    </div>
  </q-page>
</template>

<script lang="ts">
import { defineComponent, ref, computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import {
  createTimeSlot,
  getTimeSlots,
  scheduleMeeting,
  subscribeToTimeSlots,
  TimeSlot,
} from '../utils/backend';
import { date as dateUtils, useQuasar } from 'quasar';
import { selectedWallet } from 'src/utils/wallet';
import { PublicKey } from '@solana/web3.js';

function is_same_day(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() == date2.getFullYear() &&
    date1.getMonth() == date2.getMonth() &&
    date1.getDate() == date2.getDate()
  );
}

export default defineComponent({
  name: 'Index',
  components: {},
  setup() {
    const $route = useRoute();
    const $q = useQuasar();

    const calendarPubkey =
      $route.name == 'schedule'
        ? new PublicKey($route.params.id as string)
        : selectedWallet.value?.adapter.publicKey;

    const now = new Date(Date.now());
    const selectedDateString = ref(dateUtils.formatDate(now, 'YYYY/MM/DD'));

    const showAddTimeSlotDialog = ref(false);
    const addTimeSlotStartTime = ref('');
    const addTimeSlotEndTime = ref('');
    const timeSlots = ref<TimeSlot[]>([]);

    const currentDayTimeSlots = computed(() => {
      return timeSlots.value
        .filter((x) => is_same_day(selectedDate.value, x.time.start as Date))
        .sort((a, b) => a.time.start.valueOf() - b.time.start.valueOf());
    });

    watch(currentDayTimeSlots, (slots) => {
      console.log(slots.map((x) => x.id.toString()));
      console.log(
        slots.map((x) =>
          x.scheduledWith ? x.scheduledWith.toString() : 'null'
        )
      );
    });

    // Watch the selected date string and update the selected date
    const selectedDate = computed(() => {
      return new Date(Date.parse(selectedDateString.value));
    });

    if (calendarPubkey) {
      void getTimeSlots(calendarPubkey).then((slots) => {
        timeSlots.value = slots;
      });

      subscribeToTimeSlots(calendarPubkey, (timeSlot) => {
        const newTimeSlots = timeSlots.value.filter(
          (x) => !x.id.equals(timeSlot.id)
        );
        newTimeSlots.push(timeSlot);
        timeSlots.value = newTimeSlots;
      });
    }

    return {
      showAddTimeSlotDialog,
      selectedDateString,
      currentDayTimeSlots,
      addTimeSlotStartTime,
      addTimeSlotEndTime,
      $route,
      calendarPubkey,
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
          start: dateUtils.formatDate(startDate, 'YYYY/MM/DD hh:mm A'),
          end: dateUtils.formatDate(endDate, 'YYYY/MM/DD hh:mm A'),
        });

        await createTimeSlot(startDate, endDate);

        addTimeSlotStartTime.value = '';
        addTimeSlotEndTime.value = '';
      },

      scheduleMeetingButton(timeSlot: TimeSlot) {
        if ($route.name != 'schedule') {
          return;
        }
        if (!calendarPubkey) {
          return;
        }

        $q.dialog({
          title: 'Confirm',
          message: `Would you like to schedule a meeting \
          with this user at ${dateUtils.formatDate(
            timeSlot.time.start,
            'YYYY/MM/DD hh:mm A'
          )}?`,
          cancel: true,
        }).onOk(() => {
          void scheduleMeeting(timeSlot.id, 'TodoUseUsername');
        });
      },

      // Utils
      format_time(date: Date): string {
        return dateUtils.formatDate(date, 'hh:mm A');
      },
      format_pubkey(pubkey: PublicKey): string {
        let keystring = pubkey.toString();
        return (
          keystring.substr(0, 4) +
          '...' +
          keystring.substring(keystring.length - 4, keystring.length)
        );
      },
      calendarEventsHighlighter(dateStr: string) {
        const date = new Date(Date.parse(dateStr));

        return timeSlots.value.reduce((acc, slot) => {
          return is_same_day(date, slot.time.start as Date) || acc;
        }, false);
      },
    };
  },
});
</script>
