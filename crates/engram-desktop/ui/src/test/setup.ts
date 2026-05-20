import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Tauri API 모킹
vi.mock('@tauri-apps/api', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-notification', () => ({
  sendNotification: vi.fn(),
}));
