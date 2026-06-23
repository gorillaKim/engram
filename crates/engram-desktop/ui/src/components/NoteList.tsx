import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { noteList, noteGet, noteAdd, noteResolve } from '../ipc/invoke';
import { Markdown } from './Markdown';
import type { Note, NoteType } from '../ipc/types';

const NOTE_ICON: Record<NoteType, string> = {
  caveat: '⚠',
  decision: '★',
  discovery: '💡',
  blocker_detail: '🚫',
  context: '✎',
  reference: '📎',
  comment: '💬',
  evaluation: '🏆',
};

const NOTE_COLOR: Record<NoteType, string> = {
  caveat: 'text-amber-700 bg-amber-50',
  decision: 'text-indigo-700 bg-indigo-50',
  discovery: 'text-emerald-700 bg-emerald-50',
  blocker_detail: 'text-red-700 bg-red-50',
  context: 'text-slate-700 bg-slate-50',
  reference: 'text-blue-700 bg-blue-50',
  comment: 'text-slate-700 bg-slate-50',
  evaluation: 'text-purple-700 bg-purple-50',
};

const NOTE_LABEL: Record<NoteType, string> = {
  caveat: '주의',
  decision: '결정',
  discovery: '발견',
  blocker_detail: '블로커',
  context: '컨텍스트',
  reference: '참조',
  comment: '코멘트',
  evaluation: '평가',
};

// 생성 UI 에 노출할 타입 — comment 는 CommentSection 으로 분리되어 있어 제외
const CREATABLE_TYPES: NoteType[] = ['caveat', 'decision', 'discovery', 'blocker_detail', 'context', 'reference', 'evaluation'];

function NoteDetail({ id }: { id: number }) {
  const { data } = useQuery({
    queryKey: ['note', id],
    queryFn: () => noteGet(id),
  });
  if (!data?.detail) return null;
  return (
    <div className="mt-1.5 pl-5 text-xs border-l-2 border-slate-200 ml-2">
      <Markdown>{data.detail}</Markdown>
    </div>
  );
}

interface Props {
  issueId: number;
}

export function NoteList({ issueId }: Props) {
  const qc = useQueryClient();
  const [expanded, setExpanded] = useState<number | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [newType, setNewType] = useState<NoteType>('caveat');
  const [newSummary, setNewSummary] = useState('');
  const [newDetail, setNewDetail] = useState('');

  const { data: notes = [] } = useQuery({
    queryKey: ['notes', issueId],
    queryFn: () => noteList(issueId),
  });

  const add = useMutation({
    mutationFn: () =>
      noteAdd({
        issue_id: issueId,
        note_type: newType,
        summary: newSummary.trim(),
        detail: newDetail.trim() || undefined,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notes', issueId] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      setNewSummary('');
      setNewDetail('');
      setShowForm(false);
      toast.success('노트가 추가되었습니다');
    },
    onError: (err) => toast.error(`노트 추가 실패: ${err}`),
  });

  const resolve = useMutation({
    mutationFn: (id: number) => noteResolve(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notes', issueId] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('노트가 해결됨으로 표시되었습니다');
    },
    onError: (err) => toast.error(`해결 처리 실패: ${err}`),
  });

  // Exclude 'comment' notes — those are rendered in CommentSection.
  const active = notes.filter((n: Note) => !n.resolved && n.note_type !== 'comment');

  const canSubmit = newSummary.trim().length > 0;

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
          노트 ({active.length})
        </span>
        <button
          type="button"
          onClick={() => setShowForm((v) => !v)}
          className="text-xs px-2 py-0.5 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded"
        >
          {showForm ? '취소' : '+ 노트'}
        </button>
      </div>

      {active.length === 0 && !showForm && (
        <p className="text-xs text-slate-400">노트 없음</p>
      )}

      {active.map((note: Note) => (
        <div key={note.id} className="group">
          <div className={`flex items-start gap-1.5 rounded px-2 py-1.5 ${NOTE_COLOR[note.note_type]}`}>
            <button
              className="flex items-start gap-1.5 text-left flex-1 min-w-0 hover:opacity-80"
              onClick={() => setExpanded(expanded === note.id ? null : note.id)}
            >
              <span className="shrink-0">{NOTE_ICON[note.note_type]}</span>
              <span className="text-xs">{note.summary}</span>
            </button>
            <button
              type="button"
              onClick={() => resolve.mutate(note.id)}
              title="해결됨으로 표시"
              className="text-xs opacity-0 group-hover:opacity-100 hover:underline"
            >
              ✓ 해결
            </button>
          </div>
          {expanded === note.id && <NoteDetail id={note.id} />}
        </div>
      ))}

      {showForm && (
        <div className="border border-slate-200 rounded p-2 mt-2 space-y-2 bg-white">
          <div className="flex gap-2">
            <select
              value={newType}
              onChange={(e) => setNewType(e.target.value as NoteType)}
              className="text-xs px-2 py-1 border border-slate-200 rounded focus:outline-none focus:border-indigo-400"
            >
              {CREATABLE_TYPES.map((t) => (
                <option key={t} value={t}>
                  {NOTE_ICON[t]} {NOTE_LABEL[t]}
                </option>
              ))}
            </select>
            <input
              autoFocus
              value={newSummary}
              onChange={(e) => setNewSummary(e.target.value)}
              placeholder="한 줄 요약"
              className="flex-1 text-xs px-2 py-1 border border-slate-200 rounded focus:outline-none focus:border-indigo-400"
            />
          </div>
          <textarea
            value={newDetail}
            onChange={(e) => setNewDetail(e.target.value)}
            rows={2}
            placeholder="상세 (선택)"
            className="w-full text-xs px-2 py-1 border border-slate-200 rounded focus:outline-none focus:border-indigo-400 resize-none"
          />
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => { setShowForm(false); setNewSummary(''); setNewDetail(''); }}
              className="text-xs px-2 py-1 text-slate-500 hover:text-slate-700"
            >
              취소
            </button>
            <button
              type="button"
              disabled={!canSubmit || add.isPending}
              onClick={() => add.mutate()}
              className="text-xs px-3 py-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded disabled:opacity-50"
            >
              {add.isPending ? '추가 중…' : '추가'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
