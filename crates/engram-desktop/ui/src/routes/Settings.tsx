import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { getAppVersion, getActivitySettings, setActivitySettings, getPromptSettings, setPromptSettings } from '../ipc/invoke';
import {
  checkForUpdates,
  downloadAndInstall,
  relaunchApp,
} from '../services/updateManager';
import type { Update } from '../services/updateManager';
import { Info, Clock, RefreshCw, Save, CheckCircle, AlertTriangle, AppWindow, Sparkles, RotateCcw, ExternalLink } from 'lucide-react';

type UpdateState =
  | 'idle'
  | 'checking'
  | 'available'
  | 'up-to-date'
  | 'downloading'
  | 'installed'
  | 'error';

type SettingSection = 'general' | 'prompt' | 'activity' | 'update';

export function Settings() {
  const [version, setVersion] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');
  const [updateInfo, setUpdateInfo] = useState<Update | null>(null);
  const [progress, setProgress] = useState(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [warnMin, setWarnMin] = useState(30);
  const [stallMin, setStallMin] = useState(120);
  const [activitySaving, setActivitySaving] = useState(false);
  const [promptSubTab, setPromptSubTab] = useState<'issue' | 'epic' | 'mission'>('issue');
  const [issueTemplate, setIssueTemplate] = useState('{{base prompt}}');
  const [epicTemplate, setEpicTemplate] = useState('{{base prompt}}');
  const [missionTemplate, setMissionTemplate] = useState('{{base prompt}}');
  const [promptSaving, setPromptSaving] = useState(false);
  const [activeSection, setActiveSection] = useState<SettingSection>('general');

  useEffect(() => {
    getAppVersion().then(setVersion).catch(() => setVersion('unknown'));
    getActivitySettings().then(s => { setWarnMin(s.warn_minutes); setStallMin(s.stall_minutes); }).catch(() => {});
    getPromptSettings()
      .then((s) => {
        setIssueTemplate(s.issue_template || '{{base prompt}}');
        setEpicTemplate(s.epic_template || '{{base prompt}}');
        setMissionTemplate(s.mission_template || '{{base prompt}}');
      })
      .catch(() => {});
  }, []);

  async function handleSaveActivity() {
    setActivitySaving(true);
    try {
      await setActivitySettings(warnMin, stallMin);
      toast.success('설정이 정상적으로 저장되었습니다.');
    } catch {
      toast.error('설정 저장에 실패했습니다.');
    } finally {
      setActivitySaving(false);
    }
  }

  async function handleSavePrompt() {
    setPromptSaving(true);
    try {
      await setPromptSettings({
        issue_template: issueTemplate,
        epic_template: epicTemplate,
        mission_template: missionTemplate,
      });
      toast.success('프롬프트 템플릿 설정이 정상적으로 저장되었습니다.');
    } catch {
      toast.error('프롬프트 템플릿 저장에 실패했습니다.');
    } finally {
      setPromptSaving(false);
    }
  }

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

  function handleOpenReleaseNotes() {
    window.open('https://github.com/gorillaKim/engram/releases', '_blank');
  }

  return (
    <div className="flex h-full bg-slate-50/50 overflow-hidden animate-fade-in">
      {/* 1. 좌측 사이드바 */}
      <aside className="w-80 border-r border-slate-200/80 bg-white flex flex-col flex-shrink-0">
        {/* 상단 타이틀 */}
        <div className="p-4 border-b border-slate-100 flex items-center gap-2">
          <AppWindow className="w-5 h-5 text-indigo-500" />
          <h2 className="text-sm font-bold text-slate-800">앱 설정</h2>
        </div>

        {/* 설정 메뉴 목록 */}
        <div className="flex-1 overflow-y-auto p-3 space-y-1">
          {[
            { id: 'general', label: '일반 정보 및 릴리즈 노트', icon: <Info className="w-4 h-4" /> },
            { id: 'prompt', label: '프롬프트 템플릿 설정', icon: <Sparkles className="w-4 h-4" /> },
            { id: 'activity', label: '활동 임계값 설정', icon: <Clock className="w-4 h-4" /> },
            { id: 'update', label: '업데이트 확인', icon: <RefreshCw className="w-4 h-4" /> },
          ].map((sec) => (
            <button
              key={sec.id}
              onClick={() => setActiveSection(sec.id as SettingSection)}
              className={`w-full text-left px-4 py-3 rounded-xl transition-all flex items-center gap-3 text-[13px] border ${
                activeSection === sec.id
                  ? 'bg-indigo-50/80 text-indigo-600 font-semibold border-indigo-100/55 shadow-sm'
                  : 'text-slate-600 border-transparent hover:bg-slate-50 hover:text-slate-800'
              }`}
            >
              {sec.icon}
              <span className="flex-1 truncate">{sec.label}</span>
            </button>
          ))}
        </div>

        {/* 사이드바 하단 정보 */}
        <div className="p-4 bg-slate-50/60 border-t border-slate-100 flex items-center gap-3">
          <Info className="w-5 h-5 text-indigo-500 flex-shrink-0" />
          <div className="text-[11px] text-slate-500 leading-normal">
            도움이 더 필요하신가요? 터미널에서 <code className="bg-slate-200/80 px-1 rounded font-mono font-bold text-slate-700">engram --help</code> 명령어를 입력해 보세요.
          </div>
        </div>
      </aside>

      {/* 2. 우측 상세 본문 */}
      <main className="flex-1 overflow-y-auto bg-white p-8 md:p-12">
        <div className="max-w-xl mx-auto">
          {activeSection === 'general' && (
            <div className="space-y-6 animate-fade-in">
              <div className="border-b border-slate-100 pb-4">
                <h1 className="text-2xl font-bold text-slate-900 mb-1">일반 정보 및 릴리즈 노트</h1>
                <p className="text-sm text-slate-500">Engram 애플리케이션의 기본 사양, 버전 및 릴리즈 노트 정보입니다.</p>
              </div>

              <div className="bg-slate-50 border border-slate-200/60 rounded-2xl p-5 flex flex-col gap-4 shadow-sm">
                <div className="flex items-center gap-3">
                  <div className="w-12 h-12 rounded-xl bg-indigo-50 flex items-center justify-center text-indigo-600 font-bold text-lg border border-indigo-100/50 shadow-inner">
                    E
                  </div>
                  <div>
                    <h3 className="text-sm font-bold text-slate-800">Engram Desktop</h3>
                    <p className="text-xs text-slate-400">Agent Issue Management System</p>
                  </div>
                </div>

                <hr className="border-slate-200/55" />

                <div className="flex items-center justify-between text-xs text-slate-600">
                  <span className="font-medium">애플리케이션 버전</span>
                  <span className="font-mono bg-white px-2.5 py-1 rounded-lg border border-slate-200 font-bold text-slate-700 shadow-sm">
                    v{version ?? '...'}
                  </span>
                </div>

                <hr className="border-slate-200/55" />

                {/* 릴리즈 노트 바로가기 영역 */}
                <div className="flex items-center justify-between p-3.5 rounded-xl bg-indigo-50/60 border border-indigo-100/80">
                  <div className="flex flex-col gap-0.5">
                    <span className="text-xs font-bold text-indigo-950">버전별 릴리즈 노트 열람</span>
                    <span className="text-[11px] text-slate-500">배포시마다 업데이트된 기능과 버그 수정 내역을 브라우저에서 확인합니다.</span>
                  </div>
                  <button
                    onClick={handleOpenReleaseNotes}
                    className="flex items-center gap-1.5 px-3.5 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white font-semibold text-xs shadow-xs transition-all shrink-0 cursor-pointer"
                  >
                    <span>🌐 GitHub 릴리즈 노트 보기</span>
                    <ExternalLink className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeSection === 'prompt' && (
            <div className="space-y-6 animate-fade-in">
              <div className="border-b border-slate-100 pb-4">
                <h1 className="text-2xl font-bold text-slate-900 mb-1">작업 프롬프트 템플릿</h1>
                <p className="text-sm text-slate-500">
                  Prompt 버튼 클릭 시 복사되는 작업 요청 프롬프트의 양식을 이슈, 에픽, 미션 단위로 개별 설정합니다.
                </p>
              </div>

              {/* 이슈 / 에픽 / 미션 서브 탭 */}
              <div className="flex border-b border-slate-200 gap-2">
                {[
                  { id: 'issue', label: '📌 이슈 (Issue) Prompt' },
                  { id: 'epic', label: '📦 에픽 (Epic) Prompt' },
                  { id: 'mission', label: '🎯 미션 (Mission) Prompt' },
                ].map((tab) => (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setPromptSubTab(tab.id as 'issue' | 'epic' | 'mission')}
                    className={`px-4 py-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                      promptSubTab === tab.id
                        ? 'border-indigo-600 text-indigo-600 bg-indigo-50/40 rounded-t-lg'
                        : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>

              <div className="bg-slate-50 border border-slate-200/60 rounded-2xl p-5 flex flex-col gap-5 shadow-sm">
                <p className="text-xs text-slate-500 leading-relaxed bg-white p-3 rounded-xl border border-slate-150 shadow-inner">
                  <code className="bg-indigo-50 border border-indigo-200 px-1 py-0.5 rounded text-indigo-700 font-mono font-semibold">
                    {'{{base prompt}}'}
                  </code>는 선택한 단위(
                  {promptSubTab === 'issue' ? '이슈' : promptSubTab === 'epic' ? '에픽' : '미션'}
                  )의 기본 생성 문구로 자동 치환됩니다.
                </p>

                <div className="flex flex-col gap-2">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-slate-700">
                      {promptSubTab === 'issue' ? '이슈' : promptSubTab === 'epic' ? '에픽' : '미션'} 템플릿 작성
                    </span>
                    <button
                      type="button"
                      onClick={() => {
                        if (promptSubTab === 'issue') setIssueTemplate('{{base prompt}}');
                        else if (promptSubTab === 'epic') setEpicTemplate('{{base prompt}}');
                        else setMissionTemplate('{{base prompt}}');
                      }}
                      className="text-[11px] text-slate-400 hover:text-slate-600 flex items-center gap-1 transition-colors cursor-pointer"
                    >
                      <RotateCcw className="w-3 h-3" />
                      기본값으로 초기화
                    </button>
                  </div>
                  <textarea
                    rows={6}
                    value={
                      promptSubTab === 'issue'
                        ? issueTemplate
                        : promptSubTab === 'epic'
                        ? epicTemplate
                        : missionTemplate
                    }
                    onChange={(e) => {
                      const val = e.target.value;
                      if (promptSubTab === 'issue') setIssueTemplate(val);
                      else if (promptSubTab === 'epic') setEpicTemplate(val);
                      else setMissionTemplate(val);
                    }}
                    placeholder="{{base prompt}}"
                    className="w-full text-xs font-mono border border-slate-200 rounded-xl p-3 focus:ring-2 focus:ring-indigo-500/20 text-slate-700 bg-white focus:outline-none transition-all shadow-sm leading-relaxed resize-y"
                  />
                </div>

                {/* 실시간 미리보기 */}
                <div className="flex flex-col gap-2 pt-1 border-t border-slate-200/60">
                  <span className="text-xs font-semibold text-slate-700">
                    실시간 미리보기 예시 ({promptSubTab === 'issue' ? '이슈' : promptSubTab === 'epic' ? '에픽' : '미션'} 복사 시)
                  </span>
                  <div className="font-mono bg-slate-900 text-slate-200 p-3.5 rounded-xl border border-slate-800 text-[11px] leading-relaxed whitespace-pre-wrap max-h-48 overflow-y-auto shadow-inner">
                    {(promptSubTab === 'issue'
                      ? issueTemplate
                      : promptSubTab === 'epic'
                      ? epicTemplate
                      : missionTemplate
                    )
                      .split('{{base prompt}}')
                      .join(
                        promptSubTab === 'issue'
                          ? '[engram issue-#12] "소셜 로그인 UI 구현" 이슈 작업을 진행해줘. (목표: 카카오/구글 로그인 버튼 렌더링)'
                          : promptSubTab === 'epic'
                          ? '[engram epic-#45] "사용자 인증 파이프라인 고도화" 에픽 하위 이슈 작업을 진행해줘.'
                          : '[engram mission-#3] "2026 Q3 전사 보안 및 인증 강화" 미션 작업을 진행해줘.'
                      )}
                  </div>
                </div>

                <button
                  onClick={handleSavePrompt}
                  disabled={promptSaving}
                  className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-indigo-600 hover:bg-indigo-700 disabled:bg-slate-400 text-white disabled:opacity-55 transition-all shadow-sm cursor-pointer"
                >
                  <Save className="w-3.5 h-3.5" />
                  {promptSaving ? '저장 중…' : '모든 템플릿 저장'}
                </button>
              </div>
            </div>
          )}

          {activeSection === 'activity' && (
            <div className="space-y-6 animate-fade-in">
              <div className="border-b border-slate-100 pb-4">
                <h1 className="text-2xl font-bold text-slate-900 mb-1">활동 상태 임계값</h1>
                <p className="text-sm text-slate-500">이슈 및 태스크(Working 상태)의 작업 갱신이 중단되었는지를 감지하는 설정입니다.</p>
              </div>

              <div className="bg-slate-50 border border-slate-200/60 rounded-2xl p-5 flex flex-col gap-5 shadow-sm">
                <p className="text-xs text-slate-500 leading-relaxed bg-white p-3 rounded-xl border border-slate-150 shadow-inner">
                  지정된 시간(분) 동안 작업 히스토리에 변경이 없으면, 대시보드 및 트레이에서 이슈의 상태가 **작업예상 ⏸ (경고)** 또는 **작업중단 ⚠ (중단)**으로 시각화됩니다.
                </p>

                <div className="grid grid-cols-2 gap-4">
                  <div className="flex flex-col gap-1.5">
                    <span className="text-xs font-semibold text-slate-600 pl-0.5">경고 기준 (분)</span>
                    <input
                      type="number"
                      min={1}
                      value={warnMin}
                      onChange={(e) => setWarnMin(Number(e.target.value))}
                      className="w-full text-xs border border-slate-200 rounded-xl px-3 py-2 focus:ring-2 focus:ring-indigo-500/20 text-slate-700 bg-white focus:outline-none transition-all shadow-sm"
                    />
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <span className="text-xs font-semibold text-slate-600 pl-0.5">중단 기준 (분)</span>
                    <input
                      type="number"
                      min={1}
                      value={stallMin}
                      onChange={(e) => setStallMin(Number(e.target.value))}
                      className="w-full text-xs border border-slate-200 rounded-xl px-3 py-2 focus:ring-2 focus:ring-indigo-500/20 text-slate-700 bg-white focus:outline-none transition-all shadow-sm"
                    />
                  </div>
                </div>

                <button
                  onClick={handleSaveActivity}
                  disabled={activitySaving}
                  className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-indigo-600 hover:bg-indigo-700 disabled:bg-slate-400 text-white disabled:opacity-55 transition-all shadow-sm"
                >
                  <Save className="w-3.5 h-3.5" />
                  {activitySaving ? '저장 중…' : '설정 저장'}
                </button>
              </div>
            </div>
          )}

          {activeSection === 'update' && (
            <div className="space-y-6 animate-fade-in">
              <div className="border-b border-slate-100 pb-4">
                <h1 className="text-2xl font-bold text-slate-900 mb-1">앱 업데이트</h1>
                <p className="text-sm text-slate-500">최신 릴리즈 버전을 확인하고 데스크톱 앱을 업데이트합니다.</p>
              </div>

              <div className="bg-slate-50 border border-slate-200/60 rounded-2xl p-5 flex flex-col gap-4 shadow-sm">
                {updateState === 'idle' && (
                  <div className="flex flex-col gap-3">
                    <p className="text-xs text-slate-500">현재 버전 v{version ?? '...'}을 사용 중입니다.</p>
                    <div className="flex items-center gap-3 flex-wrap">
                      <button
                        onClick={handleCheck}
                        className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white transition-all shadow-sm"
                      >
                        <RefreshCw className="w-3.5 h-3.5" />
                        업데이트 지금 확인
                      </button>
                      <button
                        onClick={handleOpenReleaseNotes}
                        className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-white border border-slate-200 hover:bg-slate-100 text-slate-700 transition-all shadow-2xs"
                      >
                        <span>🌐 릴리즈 노트 열기</span>
                        <ExternalLink className="w-3.5 h-3.5 text-slate-500" />
                      </button>
                    </div>
                  </div>
                )}

                {updateState === 'checking' && (
                  <div className="flex items-center gap-2 p-3 text-xs text-slate-500 bg-white border border-slate-200/50 rounded-xl shadow-inner">
                    <RefreshCw className="w-4 h-4 text-indigo-500 animate-spin" />
                    최신 업데이트를 확인하는 중입니다…
                  </div>
                )}

                {updateState === 'up-to-date' && (
                  <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2 p-3 text-xs text-emerald-600 bg-emerald-50/50 border border-emerald-100/50 rounded-xl">
                      <CheckCircle className="w-4 h-4 text-emerald-500" />
                      현재 최신 버전을 사용하고 있습니다!
                    </div>
                    <div className="flex items-center gap-3">
                      <button
                        onClick={() => setUpdateState('idle')}
                        className="w-fit text-[11px] font-bold text-slate-400 hover:text-slate-600 transition-colors pl-1"
                      >
                        다시 확인
                      </button>
                      <button
                        onClick={handleOpenReleaseNotes}
                        className="w-fit flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-indigo-600 hover:text-indigo-800 transition-colors"
                      >
                        <span>🌐 릴리즈 노트 보기</span>
                        <ExternalLink className="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                )}

                {updateState === 'available' && updateInfo && (
                  <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2 p-3 text-xs text-amber-700 bg-amber-50/50 border border-amber-100/50 rounded-xl">
                      <AlertTriangle className="w-4 h-4 text-amber-500" />
                      <div>
                        새로운 버전을 사용할 수 있습니다: <strong className="font-semibold text-slate-800 font-mono text-[11px]">v{updateInfo.version}</strong>
                      </div>
                    </div>
                    {updateInfo.body && (
                      <div className="bg-white p-3 rounded-xl border border-slate-200 max-h-40 overflow-y-auto text-[11px] text-slate-500 leading-normal whitespace-pre-wrap shadow-inner font-mono">
                        {updateInfo.body}
                      </div>
                    )}
                    <div className="flex items-center gap-3">
                      <button
                        onClick={handleInstall}
                        className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white transition-all shadow-sm"
                      >
                        지금 설치 및 업데이트
                      </button>
                      <button
                        onClick={handleOpenReleaseNotes}
                        className="w-fit flex items-center gap-1.5 px-3.5 py-2 text-xs font-semibold rounded-xl bg-white border border-slate-200 hover:bg-slate-100 text-slate-700 transition-all shadow-2xs"
                      >
                        <span>🌐 릴리즈 노트 열기</span>
                        <ExternalLink className="w-3.5 h-3.5 text-slate-500" />
                      </button>
                    </div>
                  </div>
                )}

                {updateState === 'downloading' && (
                  <div className="flex flex-col gap-3 p-1">
                    <div className="flex items-center justify-between text-xs font-semibold text-slate-600">
                      <span>최신 업데이트 다운로드 중…</span>
                      <span className="font-mono text-indigo-600 font-bold">{progress}%</span>
                    </div>
                    <div className="h-2 w-full rounded-full bg-slate-200 overflow-hidden shadow-inner border border-slate-300/30">
                      <div
                        className="h-full rounded-full bg-indigo-600 transition-all duration-200"
                        style={{ width: `${progress}%` }}
                      />
                    </div>
                  </div>
                )}

                {updateState === 'installed' && (
                  <div className="flex flex-col gap-3">
                    <div className="p-3 text-xs text-emerald-600 bg-emerald-50/50 border border-emerald-100/50 rounded-xl flex flex-col gap-1.5">
                      <div className="flex items-center gap-2">
                        <CheckCircle className="w-4 h-4 text-emerald-500" />
                        <strong>설치가 완료되었습니다!</strong>
                      </div>
                      <p className="text-[11px] text-slate-500 pl-6">새로운 버전을 적용하려면 앱을 재시작해야 합니다.</p>
                    </div>
                    <button
                      onClick={handleRelaunch}
                      className="w-fit flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white transition-all shadow-sm"
                    >
                      지금 재시작
                    </button>
                  </div>
                )}

                {updateState === 'error' && (
                  <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2 p-3 text-xs text-red-600 bg-red-50/50 border border-red-100/50 rounded-xl">
                      <AlertTriangle className="w-4 h-4 text-red-500" />
                      오류 발생: {errorMsg}
                    </div>
                    <button
                      onClick={() => setUpdateState('idle')}
                      className="w-fit text-[11px] font-bold text-slate-400 hover:text-slate-600 transition-colors pl-1"
                    >
                      다시 시도
                    </button>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
