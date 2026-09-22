import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMemo } from "react";

export const chairStatusSchema = z.enum([
  "NotActive",
  "OnRope",
  "MountainStation",
  "ValleyStation",
]);

export type ChairStatus = z.infer<typeof chairStatusSchema>;

export const chairEventSchema = z.object({
  id: z.number(),
  status: chairStatusSchema,
  position_pulses: z.number(),
  position_meters: z.number(),
});

export const stateEventSchema = eventSchema(
  z.object({
    rope_position_pulses: z.number(),
    rope_length_pulses: z.number(),
    rope_position_meters: z.number(),
    valley_to_mountain_pulses: z.number(),
    mountain_to_valley_pulses: z.number(),
    mountain_station_pulses: z.number(),
    valley_station_pulses: z.number(),
    chairs: z.array(chairEventSchema),
  }),
);

export type StateEvent = z.infer<typeof stateEventSchema>;

export type ChairliftNamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
};

const createChairliftNamespaceStore = (): StoreApi<ChairliftNamespaceStore> =>
  create<ChairliftNamespaceStore>(() => {
    return {
      state: null,
      defaultState: null,
    };
  });

function chairliftMessageHandler(
  store: StoreApi<ChairliftNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<ChairliftNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    const updateStore = (
      updater: (state: ChairliftNamespaceStore) => ChairliftNamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          state: stateEvent,
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (e) {
      console.error(e);
    }
  };
}

const useChairliftNamespaceImplementation =
  createNamespaceHookImplementation<ChairliftNamespaceStore>({
    createStore: createChairliftNamespaceStore,
    createEventHandler: chairliftMessageHandler,
  });

export function useChairliftNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): ChairliftNamespaceStore {
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  return useChairliftNamespaceImplementation(namespaceId);
}
