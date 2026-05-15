import { create } from 'zustand';

export interface NotificationEntry {
  id: string;
  title: string;
  body: string;
  ts: number;  // Date.now()
}

interface NotificationState {
  log: NotificationEntry[];
  add: (entry: Omit<NotificationEntry, 'ts'>) => void;
  clear: () => void;
}

export const useNotificationStore = create<NotificationState>((set) => ({
  log: [],
  add: (entry) =>
    set((s) => ({
      log: [{ ...entry, ts: Date.now() }, ...s.log].slice(0, 50),
    })),
  clear: () => set({ log: [] }),
}));
