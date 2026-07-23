import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { useMcpStatus } from '../hooks/useMcpStatus';
import { useMcpCalls } from '../hooks/useMcpCalls';
import { useMcpLogs } from '../hooks/useMcpLogs';
import { useMcpTools } from '../hooks/useMcpTools';
import { mcpStart, mcpStop, mcpRestart, mcpSetAutostart } from '../ipc/invoke';
import type { SupervisorStatusSnapshot } from '../ipc/types';
import { Activity, Cpu, Terminal, Play, Square, RotateCw, Copy, Settings, Search, Info, FileText } from 'lucide-react';

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

function LevelBadge({ level }: { level: string }) {
  const cls =
    level === 'ERROR' ? 'bg-red-500/10 text-red-500 border border-red-500/20' :
    level === 'WARN'  ? 'bg-amber-500/10 text-amber-500 border border-amber-500/20' :
    level === 'INFO'  ? 'bg-blue-500/10 text-blue-500 border border-blue-500/20' :
    'bg-slate-500/10 text-slate-400 border border-slate-500/20';
  return <span className={`font-mono text-[9px] px-1.5 py-0.5 rounded font-bold ${cls}`}>{level}</span>;
}

export function McpManager() {
  const qc = useQueryClient();
  const { data: status, isLoading } = useMcpStatus();
  const calls = useMcpCalls();
  const logs = useMcpLogs();
  const [port, setPort] = useState(3456);
  const [activeTab, setActiveTab] = useState<'status' | 'tools'>('status');
  const [search, setSearch] = useState('');
  const [selectedToolName, setSelectedToolName] = useState<string | null>(null);
  const { data: tools, isLoading: isToolsLoading } = useMcpTools();

  // Load status port
  useEffect(() => {
    if (status?.port) {
      setPort(status.port);
    }
  }, [status?.port]);

  // Set default selected tool
  useEffect(() => {
    if (tools && tools.length > 0 && !selectedToolName) {
      setSelectedToolName(tools[0].name);
    }
  }, [tools, selectedToolName]);

  const mutationOpts = {
    onSuccess: (snap: SupervisorStatusSnapshot) => {
      qc.setQueryData(['mcpStatus'], snap);
    },
    onError: (err: unknown) => toast.error(`MCP 오류: ${err}`),
  };

  const startMut   = useMutation({ mutationFn: () => mcpStart(port), ...mutationOpts });
  const stopMut    = useMutation({ mutationFn: mcpStop, ...mutationOpts });
  const restartMut = useMutation({ mutationFn: () => mcpRestart(port), ...mutationOpts });
  const autostartMut = useMutation({
    mutationFn: (on: boolean) => mcpSetAutostart(on),
  });

  const endpoint = `http://127.0.0.1:${status?.port ?? port}/mcp`;

  if (isLoading) return <div className="p-8 text-slate-400">Loading…</div>;

  const filteredTools = tools?.filter(t =>
    t.name.toLowerCase().includes(search.toLowerCase()) ||
    t.description.toLowerCase().includes(search.toLowerCase())
  ) ?? [];

  const selectedTool = tools?.find(t => t.name === selectedToolName) || null;

  return (
    <div className="flex h-full bg-slate-50/50 overflow-hidden animate-fade-in">
      {/* 1. 좌측 사이드바 */}
      <aside className="w-80 border-r border-slate-200/80 bg-white flex flex-col flex-shrink-0">
        {/* 상단 탭 전환 */}
        <div className="p-4 border-b border-slate-100">
          <div className="flex p-1 bg-slate-100 rounded-xl">
            <button
              onClick={() => setActiveTab('status')}
              className={`flex-1 flex items-center justify-center gap-2 py-2 text-xs font-semibold rounded-lg transition-all ${
                activeTab === 'status'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              <Activity className="w-4 h-4" />
              상태 및 제어
            </button>
            <button
              onClick={() => setActiveTab('tools')}
              className={`flex-1 flex items-center justify-center gap-2 py-2 text-xs font-semibold rounded-lg transition-all ${
                activeTab === 'tools'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              <Cpu className="w-4 h-4" />
              도구 명세 ({tools?.length ?? 68})
            </button>
          </div>
        </div>

        {/* 탭 내용에 따른 사이드바 컨트롤 영역 */}
        <div className="flex-1 overflow-y-auto p-3 space-y-1">
          {activeTab === 'status' ? (
            <div className="space-y-4 p-2">
              {/* 상태 요약 */}
              <div className="bg-slate-50 border border-slate-200/50 rounded-xl p-3.5 flex flex-col gap-1.5 shadow-sm">
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full ${status?.running ? 'bg-green-500 animate-pulse' : 'bg-slate-400'}`} />
                  <span className="text-xs font-bold text-slate-700">
                    {status?.running ? '작동 중 (Running)' : '중지됨 (Stopped)'}
                  </span>
                </div>
                {status?.running && (
                  <div className="text-[11px] text-slate-400">
                    Uptime: {formatUptime(status.uptime_secs)}
                  </div>
                )}
              </div>

              {/* 제어 버튼 */}
              <div className="space-y-2">
                <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-1">
                  제어 명령
                </div>
                <button
                  onClick={() => startMut.mutate()}
                  disabled={status?.running || startMut.isPending}
                  className="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-semibold rounded-xl bg-green-600 hover:bg-green-700 text-white disabled:opacity-40 transition-all shadow-sm"
                >
                  <Play className="w-3.5 h-3.5" />
                  서버 시작
                </button>
                <button
                  onClick={() => stopMut.mutate()}
                  disabled={!status?.running || stopMut.isPending}
                  className="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-semibold rounded-xl bg-red-600 hover:bg-red-700 text-white disabled:opacity-40 transition-all shadow-sm"
                >
                  <Square className="w-3.5 h-3.5" />
                  서버 정지
                </button>
                <button
                  onClick={() => restartMut.mutate()}
                  disabled={restartMut.isPending}
                  className="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-semibold rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white disabled:opacity-40 transition-all shadow-sm"
                >
                  <RotateCw className="w-3.5 h-3.5" />
                  서버 재시작
                </button>
              </div>

              <hr className="border-slate-100" />

              {/* 제어 옵션 */}
              <div className="space-y-3">
                <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-1">
                  설정 옵션
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-[11px] text-slate-500 font-medium pl-0.5">포트 번호</label>
                  <input
                    type="number"
                    min={1024}
                    max={65535}
                    value={port}
                    onChange={e => {
                      const val = Number(e.target.value);
                      if (val >= 1024 && val <= 65535) setPort(val);
                    }}
                    className="w-full text-xs border border-slate-200 rounded-xl px-3 py-2 focus:ring-2 focus:ring-indigo-500/20 text-slate-700 bg-slate-50 focus:bg-white focus:outline-none transition-all"
                  />
                </div>

                <label className="flex items-center gap-2 text-xs text-slate-600 cursor-pointer p-1">
                  <input
                    type="checkbox"
                    checked={status?.autostart ?? true}
                    onChange={e => autostartMut.mutate(e.target.checked)}
                    className="rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                  />
                  앱 실행 시 자동 기동
                </label>
              </div>
            </div>
          ) : (
            // 도구 목록 탭
            <div className="space-y-4">
              {/* 도구 검색 */}
              <div className="px-1 pt-1">
                <div className="relative">
                  <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
                  <input
                    type="text"
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    placeholder="도구 검색 (예: retro, issue, sprint)..."
                    className="w-full bg-slate-100 border-none rounded-xl pl-9 pr-4 py-2 text-xs focus:ring-2 focus:ring-indigo-500/20 text-slate-700 placeholder-slate-400"
                  />
                </div>
              </div>

              {/* 도구 리스트 */}
              <div className="space-y-1">
                <div className="px-2 text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2">
                  도구 목록 ({filteredTools.length})
                </div>
                <div className="max-h-[calc(100vh-230px)] overflow-y-auto space-y-1 pr-1">
                  {isToolsLoading ? (
                    <div className="text-center py-4 text-xs text-slate-400 italic">
                      도구 로딩 중...
                    </div>
                  ) : (
                    filteredTools.map(t => (
                      <button
                        key={t.name}
                        onClick={() => setSelectedToolName(t.name)}
                        className={`w-full text-left px-3.5 py-2.5 rounded-xl transition-all flex flex-col gap-0.5 border ${
                          selectedToolName === t.name
                            ? 'bg-indigo-50/80 text-indigo-600 font-semibold border-indigo-100/55 shadow-sm'
                            : 'text-slate-600 border-transparent hover:bg-slate-50 hover:text-slate-800'
                        }`}
                      >
                        <div className="flex items-center justify-between gap-1">
                          <span className="font-mono text-xs font-bold truncate">{t.name}</span>
                          {t.name.startsWith('retro_') && (
                            <span className="px-1.5 py-0.2 rounded text-[9px] font-medium bg-emerald-50 text-emerald-700 border border-emerald-200 shrink-0">
                              회고
                            </span>
                          )}
                        </div>
                        <span className="text-[10px] text-slate-400 truncate w-full">{t.description}</span>
                      </button>
                    ))
                  )}
                  {!isToolsLoading && filteredTools.length === 0 && (
                    <div className="text-center py-8 text-xs text-slate-400 italic">
                      도구를 찾을 수 없습니다.
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* 사이드바 하단 가이드 */}
        <div className="p-4 bg-slate-50/60 border-t border-slate-100 flex items-center gap-3">
          <Info className="w-5 h-5 text-indigo-500 flex-shrink-0" />
          <div className="text-[11px] text-slate-500 leading-normal">
            도움이 더 필요하신가요? 터미널에서 <code className="bg-slate-200/80 px-1 rounded font-mono font-bold text-slate-700">engram --help</code> 명령어를 입력해 보세요.
          </div>
        </div>
      </aside>

      {/* 2. 우측 상세 본문 */}
      <main className="flex-1 overflow-y-auto bg-white p-8 md:p-12">
        <div className="max-w-3xl mx-auto">
          {activeTab === 'status' ? (
            // 상태 상세 뷰
            <div className="space-y-6 animate-fade-in">
              <div className="flex items-center justify-between border-b border-slate-100 pb-4">
                <div>
                  <h1 className="text-2xl font-bold text-slate-900 mb-1">MCP 서버 상태 및 기능 개요</h1>
                  <p className="text-sm text-slate-500">MCP(Model Context Protocol) 서버 동작 정보 및 도구 개요입니다.</p>
                </div>
                <div className="flex items-center gap-2 text-xs text-slate-500 bg-slate-50 p-2.5 rounded-xl border border-slate-200/50 shadow-sm">
                  <span className="font-semibold text-[10px] text-slate-400">엔드포인트:</span>
                  <code className="bg-white px-2 py-0.5 rounded border border-slate-200 font-mono text-slate-700 text-[10px]">{endpoint}</code>
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(endpoint);
                      toast.success('복사되었습니다.');
                    }}
                    className="text-slate-400 hover:text-indigo-600 transition-colors p-1"
                    title="복사"
                  >
                    <Copy className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* 회고(Retrospective) 및 주요 도구 기능 카드 */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-emerald-50/60 border border-emerald-100 rounded-2xl p-4 flex flex-col gap-2 shadow-2xs">
                  <div className="flex items-center gap-2 text-emerald-800 font-bold text-sm">
                    <FileText className="w-4 h-4 text-emerald-600" />
                    <span>🔄 회고 (Retrospective) 도구 8종 연동</span>
                  </div>
                  <p className="text-xs text-slate-600 leading-relaxed">
                    에이전트가 회고 작성(<code className="font-mono text-[11px] text-emerald-700">retro_create</code>), 목록 조회, 내용 업데이트 및 회고 내 액션 아이템을 미션 이슈로 일괄 변환(<code className="font-mono text-[11px] text-emerald-700">retro_action_item_convert_all_to_issues</code>)하도록 완벽히 지원합니다.
                  </p>
                </div>

                <div className="bg-indigo-50/60 border border-indigo-100 rounded-2xl p-4 flex flex-col gap-2 shadow-2xs">
                  <div className="flex items-center gap-2 text-indigo-800 font-bold text-sm">
                    <Cpu className="w-4 h-4 text-indigo-600" />
                    <span>🎯 이슈 / 에픽 / 태스크 도구 패리티</span>
                  </div>
                  <p className="text-xs text-slate-600 leading-relaxed">
                    CLI와 1:1 파리티를 보유한 68개 MCP 도구를 제공하여, 에이전트가 라이프사이클 트래킹, 차단 쿼리, 세션 복원 등을 효율적으로 수행할 수 있습니다.
                  </p>
                </div>
              </div>

              {/* Claude Code 설정 안내 */}
              <div className="bg-indigo-50/50 border border-indigo-100/80 rounded-2xl p-5 flex flex-col gap-3">
                <h3 className="text-sm font-bold text-indigo-900 flex items-center gap-1.5">
                  <Settings className="w-4 h-4 text-indigo-500" />
                  Claude Code / Agent 설정 가이드
                </h3>
                <p className="text-xs text-slate-600 leading-relaxed">
                  이 MCP 서버를 AI 에이전트에 연결하려면, 에이전트 설정 파일(예: <code className="bg-indigo-100/70 text-indigo-800 px-1 py-0.2 rounded font-mono text-[10px]">~/.claude.json</code>)의 <code className="bg-indigo-100/70 text-indigo-800 px-1 py-0.2 rounded font-mono text-[10px]">mcpServers</code> 오브젝트 내에 아래 정의를 추가하세요.
                </p>
                <pre className="bg-slate-900 text-slate-200 rounded-xl p-3.5 text-xs overflow-auto font-mono shadow-inner">
{`"mcpServers": {
  "engram": {
    "type": "http",
    "url": "${endpoint}"
  }
}`}
                </pre>
                <button
                  onClick={() => {
                    navigator.clipboard.writeText(`"engram": {\n  "type": "http",\n  "url": "${endpoint}"\n}`);
                    toast.success('설정 코드가 복사되었습니다.');
                  }}
                  className="w-fit flex items-center gap-1 text-[11px] font-bold text-indigo-600 hover:text-indigo-800 transition-colors cursor-pointer"
                >
                  설정 코드 복사하기 <Copy className="w-3 h-3" />
                </button>
              </div>

              {/* 최근 호출 리스트 */}
              <div className="space-y-3">
                <h3 className="text-sm font-bold text-slate-800 uppercase tracking-wider flex items-center gap-1.5 pl-0.5">
                  <Activity className="w-4 h-4 text-slate-500" />
                  최근 호출 ({calls.length})
                </h3>
                {calls.length === 0 ? (
                  <div className="text-center py-10 border border-dashed border-slate-200 rounded-2xl bg-slate-50/20 text-xs text-slate-400">
                    호출 내역이 없습니다.
                  </div>
                ) : (
                  <div className="max-h-[300px] overflow-auto rounded-2xl border border-slate-200 bg-white shadow-sm">
                    <table className="w-full text-xs text-left border-collapse">
                      <thead>
                        <tr className="bg-slate-50 text-slate-500 border-b border-slate-200 sticky top-0 z-10 shadow-[inset_0_-1px_0_rgba(0,0,0,0.1)]">
                          <th className="p-3 font-semibold bg-slate-50">시각</th>
                          <th className="p-3 font-semibold bg-slate-50">도구명</th>
                          <th className="p-3 font-semibold bg-slate-50">결과</th>
                          <th className="p-3 font-semibold text-right bg-slate-50">소요시간(ms)</th>
                        </tr>
                      </thead>
                      <tbody>
                        {calls.slice(0, 200).map((c, i) => (
                          <tr key={i} className="border-b border-slate-100 last:border-0 hover:bg-slate-50/50">
                            <td className="p-3 text-slate-400 font-mono">
                              {new Date(c.ts).toLocaleTimeString()}
                            </td>
                            <td className="p-3 text-slate-700 font-mono font-medium">{c.name}</td>
                            <td className="p-3">
                              <span className={c.ok ? 'text-green-600 font-semibold' : 'text-red-500 font-semibold'}>
                                {c.ok ? 'OK' : c.reason ?? 'ERR'}
                              </span>
                            </td>
                            <td className="p-3 text-right text-slate-400 font-mono">{c.duration_ms}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>

              {/* 로그 tail */}
              <div className="space-y-3">
                <h3 className="text-sm font-bold text-slate-800 uppercase tracking-wider flex items-center gap-1.5 pl-0.5">
                  <Terminal className="w-4 h-4 text-slate-500" />
                  실시간 로그 (tail)
                </h3>
                <div className="bg-slate-900 rounded-2xl p-4 text-[11px] font-mono text-slate-300 shadow-inner min-h-48 max-h-72 overflow-y-auto flex flex-col gap-1 border border-slate-800">
                  {logs.length === 0 && <span className="text-slate-500 italic pl-1">로그 데이터 없음</span>}
                  {logs.map((l, i) => (
                    <div key={i} className="flex gap-2.5 items-start hover:bg-slate-800/40 p-0.5 rounded">
                      <LevelBadge level={l.level} />
                      <span className="text-slate-500 truncate max-w-[150px] flex-shrink-0">{l.target}</span>
                      <span className="break-all whitespace-pre-wrap flex-1">{l.msg}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            // 도구 상세 정보 뷰
            <div className="animate-fade-in">
              {selectedTool ? (
                <div className="space-y-6">
                  <div className="border-b border-slate-100 pb-4">
                    <div className="flex items-center justify-between gap-2 mb-1.5">
                      <div className="flex items-center gap-2 text-xs font-semibold text-indigo-500 uppercase tracking-wider">
                        <Cpu className="w-4 h-4" />
                        MCP Tool Definition
                      </div>
                      {selectedTool.name.startsWith('retro_') && (
                        <span className="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200">
                          🔄 Retrospective 도구
                        </span>
                      )}
                    </div>
                    <h1 className="text-2xl font-bold text-slate-900 font-mono mb-2">{selectedTool.name}</h1>
                    <p className="text-sm text-slate-600 leading-relaxed bg-slate-50/50 p-3.5 rounded-2xl border border-slate-150 shadow-inner">{selectedTool.description}</p>
                  </div>

                  {/* 입력 매개변수 테이블 */}
                  <div className="space-y-3">
                    <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider pl-0.5">
                      입력 매개변수 (Input Schema)
                    </h3>
                    {Object.keys(selectedTool.inputSchema?.properties ?? {}).length === 0 ? (
                      <div className="p-5 border border-dashed border-slate-200 rounded-2xl bg-slate-50/30 text-xs text-slate-400 italic">
                        매개변수가 없는 도구입니다.
                      </div>
                    ) : (
                      <div className="overflow-x-auto rounded-2xl border border-slate-200 bg-white shadow-sm">
                        <table className="w-full text-xs text-left border-collapse">
                          <thead>
                            <tr className="bg-slate-50 text-slate-500 border-b border-slate-200">
                              <th className="p-3 font-semibold w-1/4">이름</th>
                              <th className="p-3 font-semibold w-1/6">타입</th>
                              <th className="p-3 font-semibold w-1/6">필수 여부</th>
                              <th className="p-3 font-semibold">설명</th>
                            </tr>
                          </thead>
                          <tbody>
                            {Object.entries(selectedTool.inputSchema?.properties ?? {}).map(([propName, propVal]: [string, any]) => {
                              const isReq = (selectedTool.inputSchema?.required ?? []).includes(propName);
                              return (
                                <tr key={propName} className="border-b border-slate-100 last:border-0 hover:bg-slate-50/50">
                                  <td className="p-3 font-mono font-bold text-slate-800">{propName}</td>
                                  <td className="p-3">
                                    <span className="bg-slate-100 text-slate-600 px-2 py-0.5 rounded font-mono text-[10px]">
                                      {propVal.type || 'any'}
                                    </span>
                                  </td>
                                  <td className="p-3">
                                    {isReq ? (
                                      <span className="text-red-600 font-semibold bg-red-50 px-2 py-0.5 rounded text-[10px]">필수</span>
                                    ) : (
                                      <span className="text-slate-400 bg-slate-50 px-2 py-0.5 rounded text-[10px]">선택</span>
                                    )}
                                  </td>
                                  <td className="p-3 text-slate-600 leading-normal">
                                    <p className="mb-1">{propVal.description || '-'}</p>
                                    {propVal.enum && (
                                      <div className="mt-1.5 flex flex-wrap gap-1 items-center">
                                        <span className="text-[10px] text-slate-400 font-medium">허용된 값:</span>
                                        {propVal.enum.map((ev: string) => (
                                          <span key={ev} className="bg-indigo-50 text-indigo-600 font-mono text-[10px] px-1.5 py-0.2 rounded border border-indigo-100/30">
                                            {ev}
                                          </span>
                                        ))}
                                      </div>
                                    )}
                                  </td>
                                </tr>
                              );
                            })}
                          </tbody>
                        </table>
                      </div>
                    )}
                  </div>

                  {/* 반환 데이터 형식 */}
                  <div className="space-y-3">
                    <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider pl-0.5">
                      반환 데이터 형식 (Output)
                    </h3>
                    <div className="bg-slate-50 rounded-2xl p-4 border border-slate-200/50 shadow-sm text-xs text-slate-600 leading-relaxed flex flex-col gap-2">
                      <p>
                        이 MCP 도구는 인자로 입력받는 <code className="bg-slate-200/80 px-1 py-0.2 rounded font-mono text-[10px] text-slate-700">mode</code> 파라미터 값에 따라 다음과 같은 출력 모드를 가집니다:
                      </p>
                      <ul className="list-disc pl-5 space-y-1.5">
                        <li><strong className="font-semibold text-slate-800">agent (기본값)</strong>: LLM 에이전트 해석에 최적화된 마크다운(Markdown) 문서 형식 또는 텍스트 형태로 결과를 반환합니다.</li>
                        <li><strong className="font-semibold text-slate-800">normal</strong>: 데이터의 전체 필드가 포함된 원본 JSON 형식으로 결과를 반환합니다.</li>
                        <li><strong className="font-semibold text-slate-800">compact</strong>: 요약된 정보만 한눈에 확인 가능한 가벼운 JSON 형식으로 결과를 반환합니다.</li>
                      </ul>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-center py-24 border border-dashed border-slate-200 rounded-2xl bg-slate-50/20">
                  <Cpu className="w-10 h-10 text-slate-300 mx-auto mb-3" />
                  <p className="text-sm font-medium text-slate-500">선택된 도구가 없습니다. 왼쪽 목록에서 도구를 선택하세요.</p>
                </div>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
