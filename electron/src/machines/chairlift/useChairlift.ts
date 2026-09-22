import { toastError } from "@/components/Toast";
import { chairlift1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { chairlift1SerialRoute } from "@/routes/routes";
import { useMemo } from "react";
import { useChairliftNamespace } from "./chairliftNamespace";
import z from "zod";
import { useMachineMutate } from "@/client/useClient";

export function useChairlift() {
  const { serial: serialString } = chairlift1SerialRoute.useParams();
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);
    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: {
          vendor: 0,
          machine: 0,
        },
        serial: 0,
      };
    }

    return {
      machine_identification: chairlift1.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useChairliftNamespace(machineIdentification);

  const { request: requestDeparture, isLoading: isDepartureLoading } =
    useMachineMutate(z.literal("Departure"));

  const departure = () => {
    requestDeparture({
      machine_identification_unique: machineIdentification,
      data: "Departure",
    });
  };

  return {
    state: state?.data,
    isLoading: !state,
    departure,
    isDepartureLoading,
  };
}
