import { create } from 'zustand';
import type { IssuePriority } from '../ipc/types';

type View = 'board' | 'issues' | 'mcp' | 'history';

export interface BoardFilters {
  projects: string[];          // empty = all projects
  priorities: IssuePriority[]; // empty = all priorities
  // TODO(M6): epicIds filter — UI control + applyFilters logic needed
}

interface UIState {
  view: View;
  selectedIssueId: number | null;
  selectedProjectKey: string | null;
  selectedSprintId: number | null;
  hideFinished: boolean;
  showCancelled: boolean;
  boardFilters: BoardFilters;
  setView: (v: View) => void;
  selectIssue: (id: number | null) => void;
  selectProject: (key: string | null) => void;
  selectSprint: (id: number | null) => void;
  toggleHideFinished: () => void;
  toggleShowCancelled: () => void;
  setBoardFilters: (f: Partial<BoardFilters>) => void;
  resetBoardFilters: () => void;
}

const DEFAULT_FILTERS: BoardFilters = {
  projects: [],
  priorities: [],
};

export const useUIStore = create<UIState>((set) => ({
  view: 'board',
  selectedIssueId: null,
  selectedProjectKey: null,
  selectedSprintId: null,
  hideFinished: false,
  showCancelled: false,
  boardFilters: { ...DEFAULT_FILTERS },
  setView: (view) => set({ view }),
  selectIssue: (id) => set({ selectedIssueId: id }),
  selectProject: (key) => set({ selectedProjectKey: key }),
  selectSprint: (id) => set({ selectedSprintId: id }),
  toggleHideFinished: () => set((s) => ({ hideFinished: !s.hideFinished })),
  toggleShowCancelled: () => set((s) => ({ showCancelled: !s.showCancelled })),
  setBoardFilters: (f) => set((s) => ({ boardFilters: { ...s.boardFilters, ...f } })),
  resetBoardFilters: () => set({ boardFilters: { ...DEFAULT_FILTERS } }),
}));
