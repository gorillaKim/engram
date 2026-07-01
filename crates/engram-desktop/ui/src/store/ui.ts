import { create } from 'zustand';
import type { IssuePriority } from '../ipc/types';

type View = 'board' | 'issues' | 'mcp' | 'history' | 'settings' | 'missions' | 'guide';

export interface BoardFilters {
  projects: string[];          // empty = all projects
  priorities: IssuePriority[]; // empty = all priorities
  missionIds: number[];        // empty = all missions
  epicIds: number[];           // empty = all epics
}

interface UIState {
  view: View;
  selectedIssueId: number | null;
  selectedEpicId: number | null;
  selectedMissionId: number | null;
  selectedProjectKey: string | null;
  selectedSprintId: number | null;
  hideFinished: boolean;
  showCancelled: boolean;
  boardFilters: BoardFilters;
  openedPanels: ('issue' | 'epic' | 'mission')[];
  setView: (v: View) => void;
  selectIssue: (id: number | null) => void;
  selectEpic: (id: number | null) => void;
  selectMission: (id: number | null) => void;
  selectProject: (key: string | null) => void;
  selectSprint: (id: number | null) => void;
  toggleHideFinished: () => void;
  toggleShowCancelled: () => void;
  setBoardFilters: (f: Partial<BoardFilters>) => void;
  resetBoardFilters: () => void;
  popPanel: () => void;
}

const DEFAULT_FILTERS: BoardFilters = {
  projects: [],
  priorities: [],
  missionIds: [],
  epicIds: [],
};

export const useUIStore = create<UIState>((set) => ({
  view: 'board',
  selectedIssueId: null,
  selectedEpicId: null,
  selectedMissionId: null,
  selectedProjectKey: null,
  selectedSprintId: null,
  hideFinished: false,
  showCancelled: false,
  boardFilters: { ...DEFAULT_FILTERS },
  openedPanels: [],
  setView: (view) => set({ view }),
  selectIssue: (id) => set((s) => {
    const nextPanels = id != null 
      ? [...s.openedPanels.filter(p => p !== 'issue'), 'issue'] as const
      : s.openedPanels.filter(p => p !== 'issue');
    return { selectedIssueId: id, openedPanels: nextPanels as any };
  }),
  selectEpic: (id) => set((s) => {
    const nextPanels = id != null 
      ? [...s.openedPanels.filter(p => p !== 'epic'), 'epic'] as const
      : s.openedPanels.filter(p => p !== 'epic');
    return { selectedEpicId: id, openedPanels: nextPanels as any };
  }),
  selectMission: (id) => set((s) => {
    const nextPanels = id != null 
      ? [...s.openedPanels.filter(p => p !== 'mission'), 'mission'] as const
      : s.openedPanels.filter(p => p !== 'mission');
    return { selectedMissionId: id, openedPanels: nextPanels as any };
  }),
  selectProject: (key) => set({ selectedProjectKey: key }),
  selectSprint: (id) => set({ selectedSprintId: id }),
  toggleHideFinished: () => set((s) => ({ hideFinished: !s.hideFinished })),
  toggleShowCancelled: () => set((s) => ({ showCancelled: !s.showCancelled })),
  setBoardFilters: (f) => set((s) => ({ boardFilters: { ...s.boardFilters, ...f } })),
  resetBoardFilters: () => set({ boardFilters: { ...DEFAULT_FILTERS } }),
  popPanel: () => set((s) => {
    if (s.openedPanels.length === 0) return {};
    const nextPanels = [...s.openedPanels];
    const last = nextPanels.pop();
    if (last === 'issue') return { selectedIssueId: null, openedPanels: nextPanels };
    if (last === 'epic') return { selectedEpicId: null, openedPanels: nextPanels };
    if (last === 'mission') return { selectedMissionId: null, openedPanels: nextPanels };
    return {};
  }),
}));
