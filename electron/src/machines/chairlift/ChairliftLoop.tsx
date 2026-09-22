import React from "react";
import {
  LOOP_RADIUS,
  LOOP_STRAIGHT_LENGTH,
  LoopLayout,
  pulsesToPoint,
  trackOutlinePath,
} from "./chairliftLayout";
import { ChairStatus } from "./chairliftNamespace";

type Chair = {
  id: number;
  status: ChairStatus;
  position_pulses: number;
};

type Props = {
  chairs: Chair[];
  layout: LoopLayout;
};

const statusColor: Record<ChairStatus, string> = {
  OnRope: "#2563eb",
  ValleyStation: "#16a34a",
  MountainStation: "#ea580c",
  NotActive: "#9ca3af",
};

export function ChairliftLoop({ chairs, layout }: Props) {
  const width = LOOP_STRAIGHT_LENGTH + 2 * LOOP_RADIUS;
  const height = 2 * LOOP_RADIUS;
  const padding = 28;

  const activeChairs = chairs.filter((chair) => chair.status !== "NotActive");

  return (
    <svg
      viewBox={`${-padding} ${-padding} ${width + padding * 2} ${height + padding * 2}`}
      className="h-full w-full"
    >
      <path
        d={trackOutlinePath()}
        fill="none"
        stroke="currentColor"
        strokeWidth={4}
        className="text-muted-foreground/40"
      />

      <text
        x={LOOP_RADIUS}
        y={-10}
        textAnchor="middle"
        className="fill-muted-foreground text-xs"
      >
        Talstation
      </text>
      <text
        x={LOOP_RADIUS + LOOP_STRAIGHT_LENGTH}
        y={-10}
        textAnchor="middle"
        className="fill-muted-foreground text-xs"
      >
        Bergstation
      </text>

      {activeChairs.map((chair) => {
        const { x, y } = pulsesToPoint(chair.position_pulses, layout);
        return (
          <circle
            key={chair.id}
            cx={x}
            cy={y}
            r={6}
            fill={statusColor[chair.status]}
            stroke="white"
            strokeWidth={1.5}
          >
            <title>{`Sessel ${chair.id} – ${chair.status}`}</title>
          </circle>
        );
      })}
    </svg>
  );
}
