import { useEffect, useState } from 'react';
import { Search, X, CheckCircle2, Circle, ExternalLink } from 'lucide-react';
import { issueList } from '../ipc/invoke';

export interface IssueOption {
  id: number;
  title: string;
  epic_title?: string;
  status: 'ready' | 'working' | 'demo' | 'finished';
}

interface IssueSelectModalProps {
  onSelect: (issue: IssueOption) => void;
  onClose: () => void;
}

export function IssueSelectModal({ onSelect, onClose }: IssueSelectModalProps) {
  const [search, setSearch] = useState('');
  const [issues, setIssues] = useState<IssueOption[]>([]);

  useEffect(() => {
    issueList({})
      .then((data) => {
        if (data && data.length > 0) {
          const mapped: IssueOption[] = data.map((item) => ({
            id: item.id,
            title: item.title,
            status: item.status as any,
          }));
          setIssues(mapped);
        } else {
          // Sample fallback
          setIssues([
            { id: 1188, title: 'DB Migration 0015_retrospectives.sql 추가', epic_title: '[Core & DB]', status: 'finished' },
            { id: 1189, title: 'engram-core 회고 도메인 모델 및 Repository 작성', epic_title: '[Core & DB]', status: 'finished' },
            { id: 1190, title: 'engram-mcp 회고 도구 8종 연동', epic_title: '[MCP & CLI]', status: 'finished' },
            { id: 1191, title: 'engram-cli retrospective 서브커맨드 및 파리티 확보', epic_title: '[MCP & CLI]', status: 'finished' },
            { id: 1192, title: 'Lexical 기반 리치 에디터 및 슬래시(/) 커맨드 메뉴 구현', epic_title: '[Desktop UI]', status: 'working' },
            { id: 1193, title: '회고 탭, 목록 페이지 및 상세 패널 개발', epic_title: '[Desktop UI]', status: 'working' },
          ]);
        }
      })
      .catch((err) => {
        console.warn('Failed to load issues for select modal:', err);
      });
  }, []);

  const filtered = issues.filter(
    (i) =>
      i.title.toLowerCase().includes(search.toLowerCase()) ||
      i.id.toString().includes(search) ||
      i.epic_title?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-xs">
      <div className="w-full max-w-lg bg-white border border-slate-200 rounded-xl shadow-2xl overflow-hidden flex flex-col">
        {/* 헤더 */}
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-200 bg-slate-50">
          <div className="flex items-center gap-2">
            <ExternalLink className="w-4 h-4 text-indigo-600" />
            <h3 className="font-bold text-sm text-slate-900">이슈 선택 및 에디터 삽입</h3>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-200 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* 검색 입력 */}
        <div className="p-3 border-b border-slate-100 bg-white">
          <div className="relative">
            <Search className="w-4 h-4 absolute left-3 top-2.5 text-slate-400" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="이슈 번호 (#), 제목 또는 에픽으로 검색..."
              className="w-full pl-9 pr-4 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-lg text-slate-900 placeholder-slate-400 focus:outline-none focus:border-indigo-500"
              autoFocus
            />
          </div>
        </div>

        {/* 이슈 목록 */}
        <div className="max-h-72 overflow-y-auto p-2 flex flex-col gap-1">
          {filtered.length === 0 ? (
            <div className="text-center py-6 text-slate-400 text-xs">
              검색 결과와 일치하는 이슈가 없습니다.
            </div>
          ) : (
            filtered.map((issue) => (
              <button
                key={issue.id}
                onClick={() => onSelect(issue)}
                className="flex items-center justify-between p-2.5 rounded-lg text-left hover:bg-indigo-50/80 transition-all border border-transparent hover:border-indigo-200 group"
              >
                <div className="flex items-center gap-2.5 min-w-0 flex-1">
                  {issue.status === 'finished' ? (
                    <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0" />
                  ) : (
                    <Circle className="w-4 h-4 text-indigo-500 shrink-0" />
                  )}
                  <span className="text-xs font-mono font-semibold text-slate-500 shrink-0">
                    #{issue.id}
                  </span>
                  <span className="text-xs font-medium text-slate-800 truncate group-hover:text-indigo-900">
                    {issue.title}
                  </span>
                </div>
                {issue.epic_title && (
                  <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-slate-100 text-slate-600 border border-slate-200 shrink-0 ml-2">
                    {issue.epic_title}
                  </span>
                )}
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
