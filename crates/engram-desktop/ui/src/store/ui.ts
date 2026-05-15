import { create } from 'zustand';

type View = 'board' | 'sprint' | 'mcp';

interface UIState {
  view: View;
  selectedIssueId: number | null;
  selectedProjectKey: string | null;
  hideFinished: boolean;
  setView: (v: View) => void;
  selectIssue: (id: number | null) => void;
  selectProject: (key: string | null) => void;
  toggleHideFinished: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  view: 'board',
  selectedIssueId: null,
  selectedProjectKey: null,
  hideFinished: false,
  setView: (view) => set({ view }),
  selectIssue: (id) => set({ selectedIssueId: id }),
  selectProject: (key) => set({ selectedProjectKey: key }),
  toggleHideFinished: () => set((s) => ({ hideFinished: !s.hideFinished })),
}));
