import React, { useEffect, useState } from 'react';
import { LexicalRetroEditor } from './LexicalRetroEditor';
import { IssueSelectModal, IssueOption } from './IssueSelectModal';
import { useUIStore } from '../store/ui';
import { issueList, sprintList } from '../ipc/invoke';
import { PromptButton } from './PromptButton';
import {
  CheckCircle2,
  Circle,
  ExternalLink,
  FileText,
  Link as LinkIcon,
  Plus,
  Sparkles,
  Trash2,
  Zap,
  X,
} from 'lucide-react';

export interface ActionItemUI {
  id: number;
  retro_id: number;
  title: string;
  description?: string;
  status: 'todo' | 'done' | 'converted';
  linked_issue_id?: number;
  linked_note_id?: number;
}

export interface RetrospectiveUI {
  id: number;
  project_key: string;
  sprint_name?: string;
  sprint_id?: number | null;
  title: string;
  content: string;
  created_at: string;
  updated_at: string;
  action_items: ActionItemUI[];
}

interface RetrospectiveDetailProps {
  retro: RetrospectiveUI;
  onClose: () => void;
  onUpdateContent: (content: string) => void;
  onAddActionItem: (title: string) => void;
  onToggleActionItemStatus: (itemId: number) => void;
  onDeleteActionItem: (itemId: number) => void;
  onConvertActionItem: (itemId: number) => void;
  onLinkIssueToActionItem: (itemId: number, issueId: number) => void;
  onConvertAllActionItems: () => void;
  onDeleteRetro?: () => void;
}

export function RetrospectiveDetail({
  retro,
  onClose,
  onUpdateContent,
  onAddActionItem,
  onToggleActionItemStatus,
  onDeleteActionItem,
  onConvertActionItem,
  onLinkIssueToActionItem,
  onConvertAllActionItems,
  onDeleteRetro,
}: RetrospectiveDetailProps) {
  const { selectIssue } = useUIStore();
  const [newActionTitle, setNewActionTitle] = useState('');
  const [linkingItemId, setLinkingItemId] = useState<number | null>(null);
  const [sprintStats, setSprintStats] = useState<{
    totalIssues: number;
    finishedIssues: number;
    completionRate: number;
  } | undefined>(undefined);

  useEffect(() => {
    let isMounted = true;
    const loadStats = async () => {
      try {
        const issues = await issueList({});
        let targetSprintId = retro.sprint_id;

        if (!targetSprintId && retro.sprint_name) {
          try {
            const sprints = await sprintList();
            const matched = sprints.find(
              (s) => s.name.trim().toLowerCase() === retro.sprint_name?.trim().toLowerCase()
            );
            if (matched) {
              targetSprintId = matched.id;
            }
          } catch (err) {
            console.warn('Failed to match sprint by name in detail:', err);
          }
        }

        if (!isMounted) return;

        const filtered = targetSprintId
          ? issues.filter((i) => i.sprint_id === targetSprintId)
          : issues;

        const total = filtered.length;
        const finished = filtered.filter((i) => i.status === 'finished').length;
        const rate = total > 0 ? Math.round((finished / total) * 100) : 0;

        setSprintStats({
          totalIssues: total,
          finishedIssues: finished,
          completionRate: rate,
        });
      } catch (err) {
        console.warn('Failed to calculate retro sprint stats:', err);
      }
    };

    loadStats();

    return () => {
      isMounted = false;
    };
  }, [retro.sprint_id, retro.sprint_name, retro.project_key]);

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newActionTitle.trim()) return;
    onAddActionItem(newActionTitle.trim());
    setNewActionTitle('');
  };

  const handleSelectIssueForLink = (issue: IssueOption) => {
    if (linkingItemId !== null) {
      onLinkIssueToActionItem(linkingItemId, issue.id);
      setLinkingItemId(null);
    }
  };

  const pendingCount = retro.action_items.filter((a) => !a.linked_issue_id && a.status !== 'done').length;

  return (
    <div className="flex flex-col h-full bg-white text-slate-900 shadow-2xl min-w-0">
      {/* 헤더 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-slate-200 bg-slate-50/80 shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          <div className="p-2 rounded-lg bg-indigo-50 border border-indigo-200 text-indigo-600 shrink-0">
            <FileText className="w-5 h-5" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap min-w-0">
              <span className="px-2 py-0.5 rounded text-[11px] font-mono bg-slate-100 text-slate-700 border border-slate-200 shrink-0">
                {retro.project_key}
              </span>
              {retro.sprint_name && (
                <span className="px-2 py-0.5 rounded text-[11px] font-medium bg-indigo-50 text-indigo-700 border border-indigo-200 shrink-0">
                  {retro.sprint_name}
                </span>
              )}
            </div>
            <h2 className="text-lg font-bold text-slate-900 mt-1 truncate min-w-0">{retro.title}</h2>
          </div>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          <PromptButton type="retrospective" id={retro.id} title={retro.title} size="sm" />
          {pendingCount > 0 && (
            <button
              onClick={onConvertAllActionItems}
              className="flex items-center gap-1.5 px-3.5 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white font-semibold text-xs shadow-sm transition-all active:scale-95 whitespace-nowrap shrink-0 cursor-pointer"
            >
              <Zap className="w-3.5 h-3.5 text-amber-300 fill-amber-300 shrink-0" />
              <span>Convert All to Issues ({pendingCount})</span>
            </button>
          )}
          {onDeleteRetro && (
            <button
              onClick={onDeleteRetro}
              title="회고 삭제"
              className="p-2 rounded-lg text-slate-400 hover:text-rose-600 hover:bg-rose-50 border border-transparent hover:border-rose-200 transition-colors shrink-0 cursor-pointer"
            >
              <Trash2 className="w-4.5 h-4.5" />
            </button>
          )}
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition-colors shrink-0"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* 본문 에디터 및 액션 아이템 분할 뷰 */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-4 p-6 overflow-y-auto bg-slate-50/50 min-w-0">
        {/* 좌측 2열: Lexical 에디터 */}
        <div className="lg:col-span-2 flex flex-col min-h-[480px] min-w-0">
          <LexicalRetroEditor
            value={retro.content}
            onChange={onUpdateContent}
            sprintStats={sprintStats}
            retroSprintId={retro.sprint_id}
            retroSprintName={retro.sprint_name}
          />
        </div>

        {/* 우측 1열: Action Items 패널 */}
        <div className="flex flex-col gap-4 bg-white rounded-xl p-4 border border-slate-200 shadow-sm min-w-0">
          <div className="flex items-center justify-between border-b border-slate-100 pb-3 shrink-0">
            <div className="flex items-center gap-2 min-w-0">
              <Sparkles className="w-4 h-4 text-amber-500 shrink-0" />
              <h3 className="font-semibold text-sm text-slate-800 truncate">Action Items</h3>
              <span className="px-2 py-0.5 rounded-full text-[11px] bg-slate-100 text-slate-600 font-bold shrink-0">
                {retro.action_items.length}
              </span>
            </div>
          </div>

          {/* 액션 아이템 추가 입력폼 */}
          <form onSubmit={handleAdd} className="flex gap-2 shrink-0">
            <input
              type="text"
              value={newActionTitle}
              onChange={(e) => setNewActionTitle(e.target.value)}
              placeholder="새 액션 아이템 작성..."
              className="flex-1 px-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg text-slate-900 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:bg-white min-w-0"
            />
            <button
              type="submit"
              className="px-3.5 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white text-xs font-semibold flex items-center gap-1 shrink-0 whitespace-nowrap shadow-xs cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5 shrink-0" />
              <span>추가</span>
            </button>
          </form>

          {/* 액션 아이템 리스트 */}
          <div className="flex flex-col gap-2.5 overflow-y-auto max-h-[420px] pr-1 min-w-0">
            {retro.action_items.length === 0 ? (
              <div className="text-center py-8 text-slate-400 text-xs">
                등록된 액션 아이템이 없습니다.
              </div>
            ) : (
              retro.action_items.map((item) => (
                <div
                  key={item.id}
                  className="flex flex-col gap-2 p-3 rounded-lg bg-slate-50 border border-slate-200 hover:border-slate-300 transition-all min-w-0 group"
                >
                  <div className="flex items-center justify-between gap-2 min-w-0">
                    <button
                      type="button"
                      onClick={() => onToggleActionItemStatus(item.id)}
                      className="flex items-center gap-2 min-w-0 flex-1 text-left cursor-pointer"
                    >
                      {item.status === 'done' ? (
                        <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0" />
                      ) : (
                        <Circle className="w-4 h-4 text-slate-400 group-hover:text-indigo-500 shrink-0 transition-colors" />
                      )}
                      <span
                        className={`text-xs font-medium truncate ${
                          item.status === 'done'
                            ? 'line-through text-slate-400'
                            : 'text-slate-800 group-hover:text-indigo-900'
                        }`}
                      >
                        {item.title}
                      </span>
                    </button>
                    <button
                      type="button"
                      onClick={() => onDeleteActionItem(item.id)}
                      className="p-1 rounded text-slate-400 hover:text-red-600 hover:bg-red-50 transition-colors shrink-0 cursor-pointer"
                      title="액션 아이템 삭제"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {/* 하단 이슈 연동 / 이슈 히스토리 링크 액션 버튼 */}
                  <div className="flex items-center justify-end gap-1.5 pt-1.5 border-t border-slate-200/60 shrink-0">
                    {item.linked_issue_id ? (
                      <button
                        onClick={() => selectIssue(item.linked_issue_id!)}
                        className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-indigo-50 hover:bg-indigo-100 text-indigo-700 border border-indigo-200 text-[11px] font-mono hover:border-indigo-300 transition-colors shrink-0 whitespace-nowrap cursor-pointer shadow-2xs font-semibold"
                        title="연결된 이슈 상세 정보 보기"
                      >
                        <LinkIcon className="w-3 h-3 shrink-0 text-indigo-600" />
                        <span>#{item.linked_issue_id}</span>
                        <ExternalLink className="w-3 h-3 shrink-0 text-indigo-500" />
                      </button>
                    ) : (
                      <>
                        <button
                          onClick={() => setLinkingItemId(item.id)}
                          className="flex items-center gap-1 px-2.5 py-1 rounded bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 hover:border-slate-300 text-[11px] transition-all shrink-0 whitespace-nowrap shadow-2xs cursor-pointer"
                          title="기존 이슈와 히스토리 연결"
                        >
                          <LinkIcon className="w-3 h-3 text-slate-500 shrink-0" />
                          <span>이슈 연결</span>
                        </button>
                        <button
                          onClick={() => onConvertActionItem(item.id)}
                          className="flex items-center gap-1 px-2.5 py-1 rounded bg-indigo-600 hover:bg-indigo-700 text-white border border-indigo-600 text-[11px] transition-all shrink-0 whitespace-nowrap shadow-2xs font-semibold cursor-pointer"
                          title="새 이슈로 자동 생성 및 전환"
                        >
                          <Zap className="w-3 h-3 text-amber-300 fill-amber-300 shrink-0" />
                          <span>이슈 변환</span>
                        </button>
                      </>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* 기존 이슈 연결용 모달 */}
      {linkingItemId !== null && (
        <IssueSelectModal
          onSelect={handleSelectIssueForLink}
          onClose={() => setLinkingItemId(null)}
        />
      )}
    </div>
  );
}
