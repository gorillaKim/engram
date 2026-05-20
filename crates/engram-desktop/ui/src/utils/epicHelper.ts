export function toggleAllEpics(
  epicIds: number[],
  expand: boolean
): Record<number, boolean> {
  const map: Record<number, boolean> = {};
  for (const id of epicIds) {
    map[id] = expand;
  }
  return map;
}
