export function clampSidebarWidth(
  width: number,
  min: number = 160,
  max: number = 450
): number {
  return Math.min(Math.max(width, min), max);
}
