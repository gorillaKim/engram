import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { getAppVersion } from '../ipc/invoke';
import {
  checkForUpdates,
  downloadAndInstall,
  relaunchApp,
} from '../services/updateManager';
import type { Update } from '../services/updateManager';

type UpdateState =
  | 'idle'
  | 'checking'
  | 'available'
  | 'up-to-date'
  | 'downloading'
  | 'installed'
  | 'error';

export function Settings() {
  const [version, setVersion] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');
  const [updateInfo, setUpdateInfo] = useState<Update | null>(null);
  const [progress, setProgress] = useState(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    getAppVersion().then(setVersion).catch(() => setVersion('unknown'));
  }, []);

  useEffect(() => {
    const onProgress = (e: Event) => {
      const { pct } = (e as CustomEvent<{ pct: number }>).detail;
      setProgress(pct);
    };
    const onInstalled = () => {
      setUpdateState('installed');
      setProgress(100);
    };

    window.addEventListener('update:progress', onProgress);
    window.addEventListener('update:installed', onInstalled);
    return () => {
      window.removeEventListener('update:progress', onProgress);
      window.removeEventListener('update:installed', onInstalled);
    };
  }, []);

  async function handleCheck() {
    setUpdateState('checking');
    setErrorMsg(null);
    try {
      const update = await checkForUpdates();
      if (update) {
        setUpdateInfo(update);
        setUpdateState('available');
      } else {
        setUpdateState('up-to-date');
      }
    } catch (err) {
      setUpdateState('error');
      setErrorMsg(err instanceof Error ? err.message : String(err));
      toast.error('업데이트 확인 실패');
    }
  }

  async function handleInstall() {
    setUpdateState('downloading');
    setProgress(0);
    try {
      await downloadAndInstall((pct) => setProgress(pct));
    } catch (err) {
      setUpdateState('error');
      setErrorMsg(err instanceof Error ? err.message : String(err));
      toast.error('업데이트 설치 실패');
    }
  }

  function handleRelaunch() {
    relaunchApp();
  }

  return (
    <div className="flex flex-col gap-6 p-6 max-w-xl">
      <h1 className="text-xl font-semibold text-slate-100">설정</h1>

      {/* App info */}
      <section className="rounded-lg border border-slate-700 bg-slate-800 p-4 flex flex-col gap-2">
        <h2 className="text-sm font-medium text-slate-400 uppercase tracking-wide">앱 정보</h2>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">버전</span>
          <span className="font-mono text-slate-100">{version ?? '…'}</span>
        </div>
      </section>

      {/* Update section */}
      <section className="rounded-lg border border-slate-700 bg-slate-800 p-4 flex flex-col gap-4">
        <h2 className="text-sm font-medium text-slate-400 uppercase tracking-wide">업데이트</h2>

        {updateState === 'idle' && (
          <button
            onClick={handleCheck}
            className="self-start rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
          >
            지금 확인
          </button>
        )}

        {updateState === 'checking' && (
          <p className="text-sm text-slate-400">업데이트 확인 중…</p>
        )}

        {updateState === 'up-to-date' && (
          <div className="flex flex-col gap-2">
            <p className="text-sm text-emerald-400">최신 버전입니다.</p>
            <button
              onClick={() => setUpdateState('idle')}
              className="self-start text-xs text-slate-500 hover:text-slate-300 transition-colors"
            >
              다시 확인
            </button>
          </div>
        )}

        {updateState === 'available' && updateInfo && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center gap-2">
              <span className="text-sm text-amber-400 font-medium">새 버전 사용 가능</span>
              <span className="font-mono text-xs text-slate-400">v{updateInfo.version}</span>
            </div>
            {updateInfo.body && (
              <p className="text-xs text-slate-400 whitespace-pre-wrap line-clamp-4">
                {updateInfo.body}
              </p>
            )}
            <button
              onClick={handleInstall}
              className="self-start rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
            >
              지금 설치
            </button>
          </div>
        )}

        {updateState === 'downloading' && (
          <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between text-sm text-slate-300">
              <span>다운로드 중…</span>
              <span className="font-mono">{progress}%</span>
            </div>
            <div className="h-2 w-full rounded-full bg-slate-700 overflow-hidden">
              <div
                className="h-full rounded-full bg-blue-500 transition-all duration-200"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {updateState === 'installed' && (
          <div className="flex flex-col gap-3">
            <p className="text-sm text-emerald-400">설치 완료! 변경사항을 적용하려면 재시작하세요.</p>
            <button
              onClick={handleRelaunch}
              className="self-start rounded-md bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500 transition-colors"
            >
              재시작
            </button>
          </div>
        )}

        {updateState === 'error' && (
          <div className="flex flex-col gap-2">
            <p className="text-sm text-red-400">오류: {errorMsg}</p>
            <button
              onClick={() => setUpdateState('idle')}
              className="self-start text-xs text-slate-500 hover:text-slate-300 transition-colors"
            >
              다시 시도
            </button>
          </div>
        )}
      </section>
    </div>
  );
}
