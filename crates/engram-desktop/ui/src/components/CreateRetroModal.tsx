import React, { useState } from 'react';
import { FileText, X, Sparkles, Calendar } from 'lucide-react';

export interface CreateRetroFormData {
  project_key: string;
  sprint_name: string;
  title: string;
}

interface CreateRetroModalProps {
  onCreated: (data: CreateRetroFormData) => void;
  onClose: () => void;
}

export function CreateRetroModal({ onCreated, onClose }: CreateRetroModalProps) {
  const [projectKey, setProjectKey] = useState('engram');
  const [sprintName, setSprintName] = useState('Sprint 14');
  const [customSprint, setCustomSprint] = useState('');
  const [isCustomSprint, setIsCustomSprint] = useState(false);
  const [title, setTitle] = useState('');

  // Sample sprint options
  const sprintOptions = ['Sprint 14', 'Sprint 13', 'Sprint 12', 'Sprint 11'];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const finalSprint = isCustomSprint ? customSprint.trim() : sprintName;
    const finalTitle = title.trim() || `${finalSprint} 회고`;

    if (!finalSprint) return;

    onCreated({
      project_key: projectKey,
      sprint_name: finalSprint,
      title: finalTitle,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 backdrop-blur-xs animate-fade-in">
      <div className="w-full max-w-md bg-white border border-slate-200 rounded-2xl shadow-2xl overflow-hidden flex flex-col">
        {/* 헤더 */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-100 bg-slate-50/80">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-indigo-50 border border-indigo-100 text-indigo-600">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-bold text-base text-slate-900">새 회고 작성</h3>
              <p className="text-xs text-slate-500">스프린트를 선택하고 회고를 시작합니다.</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-200/70 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* 폼 본문 */}
        <form onSubmit={handleSubmit} className="p-6 flex flex-col gap-4">
          {/* 1. 프로젝트 선택 */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-bold text-slate-700">프로젝트 키 (Project Key)</label>
            <input
              type="text"
              value={projectKey}
              onChange={(e) => setProjectKey(e.target.value)}
              className="w-full px-3.5 py-2 text-xs bg-slate-50 border border-slate-200 rounded-xl text-slate-900 font-mono focus:outline-none focus:border-indigo-500 focus:bg-white"
              required
            />
          </div>

          {/* 2. 대상 스프린트 선택 */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-bold text-slate-700 flex items-center gap-1">
              <Calendar className="w-3.5 h-3.5 text-indigo-600" />
              <span>대상 스프린트 (Sprint)</span>
            </label>

            {!isCustomSprint ? (
              <div className="flex gap-2">
                <select
                  value={sprintName}
                  onChange={(e) => {
                    if (e.target.value === '__custom__') {
                      setIsCustomSprint(true);
                    } else {
                      setSprintName(e.target.value);
                    }
                  }}
                  className="flex-1 px-3.5 py-2 text-xs bg-slate-50 border border-slate-200 rounded-xl text-slate-900 focus:outline-none focus:border-indigo-500 focus:bg-white font-medium"
                >
                  {sprintOptions.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                  <option value="__custom__">+ 직접 입력...</option>
                </select>
              </div>
            ) : (
              <div className="flex gap-2">
                <input
                  type="text"
                  value={customSprint}
                  onChange={(e) => setCustomSprint(e.target.value)}
                  placeholder="예: Sprint 15"
                  className="flex-1 px-3.5 py-2 text-xs bg-white border border-indigo-300 rounded-xl text-slate-900 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setIsCustomSprint(false)}
                  className="px-3 py-2 text-xs text-slate-500 hover:text-slate-800 bg-slate-100 rounded-xl border border-slate-200"
                >
                  목록 선택
                </button>
              </div>
            )}
          </div>

          {/* 3. 회고 제목 */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-bold text-slate-700 flex items-center gap-1">
              <FileText className="w-3.5 h-3.5 text-indigo-600" />
              <span>회고 제목</span>
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={`${isCustomSprint ? customSprint || 'Sprint' : sprintName} 회고 및 액션 아이템 수립`}
              className="w-full px-3.5 py-2 text-xs bg-slate-50 border border-slate-200 rounded-xl text-slate-900 placeholder-slate-400 focus:outline-none focus:border-indigo-500 focus:bg-white font-medium"
            />
          </div>

          {/* 버튼 하단 */}
          <div className="flex items-center justify-end gap-2.5 pt-4 border-t border-slate-100 mt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl text-xs font-medium text-slate-600 hover:bg-slate-100 border border-slate-200 transition-colors"
            >
              취소
            </button>
            <button
              type="submit"
              className="px-5 py-2 rounded-xl text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 shadow-sm transition-all active:scale-95 flex items-center gap-1.5"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>회고 작성 시작</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
