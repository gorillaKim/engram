import { check } from '@tauri-apps/plugin-updater';
import type { Update } from '@tauri-apps/plugin-updater';

export type { Update };

export interface CheckOptions {
  silent?: boolean;
}

export async function checkForUpdates(opts?: CheckOptions): Promise<Update | null> {
  const timeout = new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error('update check timed out')), 10_000),
  );

  try {
    const update = await Promise.race([check(), timeout]);
    return update ?? null;
  } catch (err) {
    if (opts?.silent) return null;
    throw err;
  }
}

export async function downloadAndInstall(
  onProgress?: (pct: number) => void,
): Promise<void> {
  const update = await check();
  if (!update) return;

  let contentLength: number | null = null;
  let downloaded = 0;

  await update.downloadAndInstall((event) => {
    if (event.event === 'Started') {
      contentLength = event.data.contentLength ?? null;
      downloaded = 0;
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      const pct =
        contentLength && contentLength > 0
          ? Math.min(Math.round((downloaded / contentLength) * 100), 99)
          : 0;
      onProgress?.(pct);
      window.dispatchEvent(
        new CustomEvent('update:progress', { detail: { pct, downloaded, contentLength } }),
      );
    } else if (event.event === 'Finished') {
      onProgress?.(100);
      window.dispatchEvent(new CustomEvent('update:installed'));
    }
  });
}

export { relaunchApp } from '../ipc/invoke';
