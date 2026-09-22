export type LoopLayout = {
  valleyToMountainPulses: number;
  mountainToValleyPulses: number;
  mountainStationPulses: number;
  valleyStationPulses: number;
};

export const LOOP_STRAIGHT_LENGTH = 520;
export const LOOP_RADIUS = 90;

export type Point = { x: number; y: number };

function mod(a: number, n: number): number {
  return ((a % n) + n) % n;
}

export function totalPulses(layout: LoopLayout): number {
  return layout.valleyToMountainPulses + layout.mountainToValleyPulses;
}

/**
 * Maps an absolute rope-pulse position to a point on the loop, using the same
 * segment layout as the backend's `ChairliftConfig::zone_at` (station zones
 * centered on the segment boundaries, straights in between). Kept as an
 * "obround"/stadium shape: two straights connected by semicircular turns.
 */
export function pulsesToPoint(pulses: number, layout: LoopLayout): Point {
  const R = LOOP_RADIUS;
  const W = LOOP_STRAIGHT_LENGTH;
  const total = totalPulses(layout);
  if (total <= 0) {
    return { x: R, y: 0 };
  }

  const valleyHalf = layout.valleyStationPulses / 2;
  const mountainHalf = layout.mountainStationPulses / 2;
  const valleyWidth = layout.valleyStationPulses;
  const mountainWidth = layout.mountainStationPulses;

  const b1 = valleyHalf;
  const b2 = layout.valleyToMountainPulses - mountainHalf;
  const b3 = layout.valleyToMountainPulses + mountainHalf;
  const b4 = total - valleyHalf;

  const p = mod(pulses, total);

  // Valley turn straddles the wrap point (p = 0), left semicircle.
  const uValley = mod(p - b4, total);
  if (uValley < valleyWidth) {
    const f = valleyWidth > 0 ? uValley / valleyWidth : 0.5;
    const theta = ((90 + f * 180) * Math.PI) / 180;
    return { x: R + R * Math.cos(theta), y: R + R * Math.sin(theta) };
  }

  // Top straight: valley -> mountain, left to right.
  if (p >= b1 && p < b2) {
    const f = b2 > b1 ? (p - b1) / (b2 - b1) : 0;
    return { x: R + f * W, y: 0 };
  }

  // Mountain turn, right semicircle.
  if (p >= b2 && p < b3) {
    const f = mountainWidth > 0 ? (p - b2) / mountainWidth : 0.5;
    const theta = ((-90 + f * 180) * Math.PI) / 180;
    return { x: R + W + R * Math.cos(theta), y: R + R * Math.sin(theta) };
  }

  // Bottom straight: mountain -> valley, right to left.
  const f = b4 > b3 ? (p - b3) / (b4 - b3) : 0;
  return { x: R + W - f * W, y: 2 * R };
}

export function trackOutlinePath(): string {
  const R = LOOP_RADIUS;
  const W = LOOP_STRAIGHT_LENGTH;
  return `M ${R} 0 H ${R + W} A ${R} ${R} 0 0 1 ${R + W} ${2 * R} H ${R} A ${R} ${R} 0 0 1 ${R} 0 Z`;
}
