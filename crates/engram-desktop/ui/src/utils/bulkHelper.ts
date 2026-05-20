export function toggleIssueSelection(selectedIds: number[], id: number): number[] {
  if (selectedIds.includes(id)) {
    return selectedIds.filter((item) => item !== id);
  }
  return [...selectedIds, id];
}

export function toggleAllIssuesInEpic(
  selectedIds: number[],
  epicIssueIds: number[],
  selectAll: boolean
): number[] {
  if (selectAll) {
    const toAdd = epicIssueIds.filter((id) => !selectedIds.includes(id));
    return [...selectedIds, ...toAdd];
  } else {
    return selectedIds.filter((id) => !epicIssueIds.includes(id));
  }
}
