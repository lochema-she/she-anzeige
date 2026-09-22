import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { TouchButton } from "@/components/touch/TouchButton";
import React from "react";
import { useChairlift } from "./useChairlift";
import { ChairliftLoop } from "./ChairliftLoop";

export function ChairliftControlPage() {
  const { state, isLoading, departure, isDepartureLoading } = useChairlift();

  const chairs = state?.chairs ?? [];
  const activeChairCount = chairs.filter((c) => c.status !== "NotActive").length;
  const waitingChairCount = chairs.length - activeChairCount;

  return (
    <Page>
      <ControlCard title="Seilposition" className="min-h-[280px]">
        {state ? (
          <ChairliftLoop
            chairs={chairs}
            layout={{
              valleyToMountainPulses: state.valley_to_mountain_pulses,
              mountainToValleyPulses: state.mountain_to_valley_pulses,
              mountainStationPulses: state.mountain_station_pulses,
              valleyStationPulses: state.valley_station_pulses,
            }}
          />
        ) : (
          <div className="text-muted-foreground flex h-full min-h-[200px] items-center justify-center italic">
            {isLoading ? "Warte auf Daten..." : "Keine Daten empfangen"}
          </div>
        )}
      </ControlCard>

      <div className="flex flex-wrap gap-6">
        <ControlCard title="Seilstatus">
          <div className="flex flex-col gap-1 text-lg">
            <span>
              Position:{" "}
              <span className="font-mono">
                {state ? `${state.rope_position_meters.toFixed(1)} m` : "–"}
              </span>
            </span>
            <span>
              Sessel auf der Strecke:{" "}
              <span className="font-mono">{activeChairCount}</span>
            </span>
            <span>
              Wartende Sessel:{" "}
              <span className="font-mono">{waitingChairCount}</span>
            </span>
          </div>
        </ControlCard>

        <ControlCard title="Abfahrt (manuell)">
          <p className="text-muted-foreground text-sm">
            Kein Abfahrtssensor vorhanden. Löst manuell die Abfahrt des
            nächsten wartenden Sessels an der aktuellen Seilposition aus.
          </p>
          <TouchButton
            variant="default"
            icon="lu:CableCar"
            className="h-16"
            onClick={() => departure()}
            disabled={isDepartureLoading || waitingChairCount === 0}
          >
            Abfahrt auslösen
          </TouchButton>
        </ControlCard>

        <ControlCard title="Legende">
          <div className="flex flex-col gap-2 text-sm">
            <LegendEntry color="#16a34a" label="In Talstation" />
            <LegendEntry color="#2563eb" label="Auf der Strecke" />
            <LegendEntry color="#ea580c" label="In Bergstation" />
          </div>
        </ControlCard>
      </div>
    </Page>
  );
}

function LegendEntry({ color, label }: { color: string; label: string }) {
  return (
    <div className="flex items-center gap-2">
      <span
        className="inline-block size-3 rounded-full"
        style={{ backgroundColor: color }}
      />
      <span>{label}</span>
    </div>
  );
}
