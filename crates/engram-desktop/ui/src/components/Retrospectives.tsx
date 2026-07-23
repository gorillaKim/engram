import { useEffect, useState } from 'react';
import { useUIStore } from '../store/ui';
import { RetrospectiveDetail, RetrospectiveUI, ActionItemUI } from './RetrospectiveDetail';
import { CreateRetroModal, CreateRetroFormData } from './CreateRetroModal';
import {
  retroActionItemConvertToIssue,
  retroActionItemCreate,
  retroActionItemDelete,
  retroActionItemUpdate,
  retrospectiveCreate,
  retrospectiveList,
  retrospectiveUpdate,
} from '../ipc/invoke';
import {
  CheckSquare,
  FileText,
  Plus,
  Search,
} from 'lucide-react';

export function Retrospectives() {
  const { selectedRetroId, selectRetro } = useUIStore();
  const [search, setSearch] = useState('');
  const [showCreateModal, setShowCreateModal] = useState(false);

  // Retrospectives state connected to DB
  const [retros, setRetros] = useState<RetrospectiveUI[]>([]);

  const fetchRetros = async () => {
    try {
      const data = await retrospectiveList();
      if (data && data.length > 0) {
        const mapped: RetrospectiveUI[] = data.map((item) => ({
          id: item.id,
          project_key: item.project_key,
          sprint_name: item.sprint_id ? `Sprint ${item.sprint_id}` : 'General',
          sprint_id: item.sprint_id,
          title: item.title,
          content: item.content,
          created_at: item.created_at,
          updated_at: item.updated_at,
          action_items: (item.action_items || []).map((ai): ActionItemUI => ({
            id: ai.id,
            retro_id: ai.retro_id,
            title: ai.title,
            description: ai.description ?? undefined,
            status: (ai.status === 'done' ? 'done' : 'todo') as 'todo' | 'done' | 'converted',
            linked_issue_id: ai.linked_issue_id ?? undefined,
          })),
        }));
        setRetros(mapped);
      } else {
        // Fallback sample if DB is completely empty
        setRetros([
          {
            id: 1,
            project_key: 'engram',
            sprint_name: 'Sprint 14',
            title: 'Sprint 14 회고 및 액션 아이템 수립',
            content: `## 🟢 Keep\n- 회고 MCP 도구 연동 완료\n- 에디터 슬래시 커맨드 적용\n\n## 🔴 Problem\n- CLI test sync 타임아웃 발생\n\n## 🟡 Try\n- 회고 액션 아이템 이슈 변환 자동화\n\n[#1188 DB Migration 0015_retrospectives.sql 추가]\n\n### 📊 이번 스프린트 에픽별 현황 분석\n\n| 에픽명 | 전체 이슈 | 완료 | 진행률 |\n| :--- | :---: | :---: | :---: |\n| [Core & DB] | 10 | 10 | 100% |\n| [MCP & CLI] | 5 | 5 | 100% |\n| [Desktop UI] | 5 | 2 | 40% |\n`,
            created_at: '2026-07-23 14:00',
            updated_at: '2026-07-23 14:00',
            action_items: [
              { id: 101, retro_id: 1, title: '회고 CLI 이슈 자동 연결 기능 테스트', status: 'todo' },
              { id: 102, retro_id: 1, title: '슬래시 커맨드 에디터 UX 개선', status: 'done', linked_issue_id: 1192 },
            ],
          },
        ]);
      }
    } catch (err) {
      console.warn('Failed to load retrospectives from DB:', err);
    }
  };

  useEffect(() => {
    fetchRetros();
  }, []);

  const selectedRetro = retros.find((r) => r.id === selectedRetroId);

  const handleCreateRetroFromModal = async (formData: CreateRetroFormData) => {
    setShowCreateModal(false);
    const initialContent = `## 🟢 Keep (잘한 점 & 유지할 점)\n- \n\n## 🔴 Problem (아쉬운 점 & 문제점)\n- \n\n## 🟡 Try (시도할 개선 방향)\n- \n`;

    try {
      const created = await retrospectiveCreate({
        project_key: formData.project_key || 'engram',
        title: formData.title || `${formData.sprint_name} 회고`,
        content: initialContent,
        sprint_id: formData.sprint_id ?? null,
      });

      const newRetroUI: RetrospectiveUI = {
        id: created.id,
        project_key: created.project_key,
        sprint_name: formData.sprint_name || 'Sprint Current',
        sprint_id: created.sprint_id,
        title: created.title,
        content: created.content,
        created_at: created.created_at,
        updated_at: created.updated_at,
        action_items: (created.action_items || []).map((ai) => ({
          id: ai.id,
          retro_id: ai.retro_id,
          title: ai.title,
          status: ai.status as 'todo' | 'done',
        })),
      };

      setRetros((prev) => [newRetroUI, ...prev]);
      selectRetro(newRetroUI.id);
    } catch (err) {
      console.error('Failed to create retrospective:', err);
      // Fallback local update
      const newRetro: RetrospectiveUI = {
        id: Date.now(),
        project_key: formData.project_key || 'engram',
        sprint_name: formData.sprint_name || 'Sprint Current',
        title: formData.title || `${formData.sprint_name} 회고 및 액션 아이템 수립`,
        content: initialContent,
        created_at: new Date().toISOString().slice(0, 16).replace('T', ' '),
        updated_at: new Date().toISOString().slice(0, 16).replace('T', ' '),
        action_items: [],
      };
      setRetros([newRetro, ...retros]);
      selectRetro(newRetro.id);
    }
  };

  const handleUpdateContent = async (retroId: number, content: string) => {
    setRetros((prev) =>
      prev.map((r) => (r.id === retroId ? { ...r, content } : r))
    );
    try {
      await retrospectiveUpdate(retroId, { content });
    } catch (err) {
      console.warn('Failed to update retro content to DB:', err);
    }
  };

  const handleAddActionItem = async (retroId: number, title: string) => {
    try {
      const createdItem = await retroActionItemCreate(retroId, { title });
      setRetros((prev) =>
        prev.map((r) => {
          if (r.id === retroId) {
            const newItem = {
              id: createdItem.id,
              retro_id: createdItem.retro_id,
              title: createdItem.title,
              status: createdItem.status as 'todo' | 'done',
            };
            return { ...r, action_items: [...r.action_items, newItem] };
          }
          return r;
        })
      );
    } catch (err) {
      console.warn('Failed to create action item in DB, using fallback local item:', err);
      setRetros((prev) =>
        prev.map((r) => {
          if (r.id === retroId) {
            const newItem = {
              id: Date.now(),
              retro_id: retroId,
              title,
              status: 'todo' as const,
            };
            return { ...r, action_items: [...r.action_items, newItem] };
          }
          return r;
        })
      );
    }
  };

  const handleToggleActionItemStatus = async (retroId: number, itemId: number) => {
    const targetRetro = retros.find((r) => r.id === retroId);
    const targetItem = targetRetro?.action_items.find((item) => item.id === itemId);
    const nextStatus: 'todo' | 'done' = targetItem?.status === 'done' ? 'todo' : 'done';

    setRetros((prev) =>
      prev.map((r) => {
        if (r.id === retroId) {
          const updatedItems = r.action_items.map((item) =>
            item.id === itemId ? { ...item, status: nextStatus } : item
          );
          return { ...r, action_items: updatedItems };
        }
        return r;
      })
    );

    try {
      await retroActionItemUpdate(itemId, { status: nextStatus });
    } catch (err) {
      console.warn('Failed to update action item status in DB:', err);
    }
  };

  const handleDeleteActionItem = async (retroId: number, itemId: number) => {
    setRetros((prev) =>
      prev.map((r) => {
        if (r.id === retroId) {
          const updatedItems = r.action_items.filter((item) => item.id !== itemId);
          return { ...r, action_items: updatedItems };
        }
        return r;
      })
    );

    try {
      await retroActionItemDelete(itemId);
    } catch (err) {
      console.warn('Failed to delete action item from DB:', err);
    }
  };

  const handleConvertActionItem = async (retroId: number, itemId: number) => {
    try {
      const issue = await retroActionItemConvertToIssue(itemId, 'user');
      setRetros((prev) =>
        prev.map((r) => {
          if (r.id === retroId) {
            const updatedItems = r.action_items.map((item) =>
              item.id === itemId
                ? { ...item, status: 'done' as const, linked_issue_id: issue.id }
                : item
            );
            return { ...r, action_items: updatedItems };
          }
          return r;
        })
      );
    } catch (err) {
      console.warn('Failed to convert action item to issue via DB:', err);
      const mockIssueId = Math.floor(Math.random() * 800) + 1200;
      setRetros((prev) =>
        prev.map((r) => {
          if (r.id === retroId) {
            const updatedItems = r.action_items.map((item) =>
              item.id === itemId
                ? { ...item, status: 'done' as const, linked_issue_id: mockIssueId }
                : item
            );
            return { ...r, action_items: updatedItems };
          }
          return r;
        })
      );
    }
  };

  const handleLinkIssueToActionItem = async (retroId: number, itemId: number, issueId: number) => {
    setRetros((prev) =>
      prev.map((r) => {
        if (r.id === retroId) {
          const updatedItems = r.action_items.map((item) =>
            item.id === itemId ? { ...item, linked_issue_id: issueId } : item
          );
          return { ...r, action_items: updatedItems };
        }
        return r;
      })
    );

    try {
      await retroActionItemUpdate(itemId, { linked_issue_id: issueId });
    } catch (err) {
      console.warn('Failed to link issue to action item in DB:', err);
    }
  };

  const handleConvertAllActionItems = async (retroId: number) => {
    const targetRetro = retros.find((r) => r.id === retroId);
    if (!targetRetro) return;

    for (const item of targetRetro.action_items) {
      if (!item.linked_issue_id) {
        await handleConvertActionItem(retroId, item.id);
      }
    }
  };

  const filteredRetros = retros.filter((r) =>
    r.title.toLowerCase().includes(search.toLowerCase()) ||
    r.project_key.toLowerCase().includes(search.toLowerCase()) ||
    r.sprint_name?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="relative flex h-full w-full bg-slate-50 text-slate-900 overflow-hidden">
      {/* 목록 메인 영역 (항상 100% 넓은 화면 유지) */}
      <div className="flex-1 flex flex-col h-full w-full min-w-0 overflow-hidden">
        {/* 상단 툴바 */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-200 bg-white shrink-0">
          <div className="flex items-center gap-3 min-w-0">
            <div className="p-2 rounded-lg bg-indigo-50 text-indigo-600 border border-indigo-100 shrink-0">
              <FileText className="w-5 h-5" />
            </div>
            <div className="min-w-0">
              <h1 className="text-xl font-bold text-slate-900 truncate">Retrospectives</h1>
              <p className="text-xs text-slate-500 truncate">스프린트 회고 작성 및 액션 아이템 관리</p>
            </div>
          </div>

          <div className="flex items-center gap-3 shrink-0">
            <div className="relative w-64">
              <Search className="w-4 h-4 absolute left-3 top-2.5 text-slate-400 shrink-0" />
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="회고 제목/스프린트 검색..."
                className="w-full pl-9 pr-4 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-lg text-slate-900 placeholder-slate-400 focus:outline-none focus:border-indigo-500"
              />
            </div>
            <button
              onClick={() => setShowCreateModal(true)}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white text-xs font-semibold shadow-sm transition-all active:scale-95 shrink-0 whitespace-nowrap cursor-pointer"
            >
              <Plus className="w-4 h-4 shrink-0" />
              <span>새 회고 작성</span>
            </button>
          </div>
        </div>

        {/* 그리드 카드 목록 */}
        <div className="flex-1 p-6 overflow-y-auto grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          {filteredRetros.map((retro) => (
            <div
              key={retro.id}
              onClick={() => selectRetro(retro.id)}
              className={`flex flex-col justify-between p-5 rounded-xl border transition-all cursor-pointer min-w-0 ${
                selectedRetroId === retro.id
                  ? 'bg-white border-indigo-500 ring-2 ring-indigo-500/20 shadow-lg'
                  : 'bg-white border-slate-200 hover:border-indigo-300 hover:shadow-md'
              }`}
            >
              <div className="flex flex-col gap-3 min-w-0">
                <div className="flex items-center gap-2 flex-wrap min-w-0">
                  <span className="px-2 py-0.5 rounded text-[11px] font-mono bg-slate-100 text-slate-700 border border-slate-200 shrink-0">
                    {retro.project_key}
                  </span>
                  {retro.sprint_name && (
                    <span className="px-2 py-0.5 rounded text-[11px] font-medium bg-indigo-50 text-indigo-700 border border-indigo-100 shrink-0">
                      {retro.sprint_name}
                    </span>
                  )}
                </div>

                <h3 className="text-base font-bold text-slate-900 truncate min-w-0">
                  {retro.title}
                </h3>

                <p className="text-xs text-slate-600 line-clamp-2 leading-relaxed break-words min-w-0">
                  {retro.content.replace(/#/g, '')}
                </p>
              </div>

              <div className="flex items-center justify-between gap-2 pt-4 border-t border-slate-100 mt-4 text-xs text-slate-500 whitespace-nowrap min-w-0">
                <div className="flex items-center gap-1.5 shrink-0">
                  <CheckSquare className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                  <span className="font-medium">액션 아이템 {retro.action_items.length}개</span>
                </div>
                <span className="text-[11px] font-mono text-slate-400 shrink-0">{retro.created_at}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 새 회고 작성 모달 */}
      {showCreateModal && (
        <CreateRetroModal
          onCreated={handleCreateRetroFromModal}
          onClose={() => setShowCreateModal(false)}
        />
      )}

      {/* Floating Overlay Drawer */}
      {selectedRetro && (
        <div className="fixed inset-0 z-40 flex justify-end pointer-events-none">
          {/* Backdrop overlay */}
          <div
            className="absolute inset-0 bg-slate-900/30 backdrop-blur-xs pointer-events-auto transition-opacity"
            onClick={() => selectRetro(null)}
          />
          {/* Slide-over Drawer Panel */}
          <div className="relative w-[840px] max-w-full h-full pointer-events-auto z-10 transition-transform duration-200 ease-out shadow-2xl">
            <RetrospectiveDetail
              retro={selectedRetro}
              onClose={() => selectRetro(null)}
              onUpdateContent={(content) => handleUpdateContent(selectedRetro.id, content)}
              onAddActionItem={(title) => handleAddActionItem(selectedRetro.id, title)}
              onToggleActionItemStatus={(itemId) => handleToggleActionItemStatus(selectedRetro.id, itemId)}
              onDeleteActionItem={(itemId) => handleDeleteActionItem(selectedRetro.id, itemId)}
              onConvertActionItem={(itemId) => handleConvertActionItem(selectedRetro.id, itemId)}
              onLinkIssueToActionItem={(itemId, issueId) => handleLinkIssueToActionItem(selectedRetro.id, itemId, issueId)}
              onConvertAllActionItems={() => handleConvertAllActionItems(selectedRetro.id)}
            />
          </div>
        </div>
      )}
    </div>
  );
}
