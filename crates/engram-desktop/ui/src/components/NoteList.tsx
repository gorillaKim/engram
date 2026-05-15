import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { noteList, noteGet } from '../ipc/invoke';
import type { Note, NoteType } from '../ipc/types';

const NOTE_ICON: Record<NoteType, string> = {
  caveat: '⚠',
  decision: '★',
  discovery: '💡',
  blocker_detail: '🚫',
  context: '✎',
  reference: '📎',
};

const NOTE_COLOR: Record<NoteType, string> = {
  caveat: 'text-amber-700 bg-amber-50',
  decision: 'text-indigo-700 bg-indigo-50',
  discovery: 'text-emerald-700 bg-emerald-50',
  blocker_detail: 'text-red-700 bg-red-50',
  context: 'text-slate-700 bg-slate-50',
  reference: 'text-blue-700 bg-blue-50',
};

function NoteDetail({ id }: { id: number }) {
  const { data } = useQuery({
    queryKey: ['note', id],
    queryFn: () => noteGet(id),
  });
  if (!data?.detail) return null;
  return (
    <p className="mt-1 text-xs text-slate-600 whitespace-pre-wrap pl-5">{data.detail}</p>
  );
}

interface Props {
  issueId: number;
}

export function NoteList({ issueId }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null);
  const { data: notes = [] } = useQuery({
    queryKey: ['notes', issueId],
    queryFn: () => noteList(issueId),
  });

  const active = notes.filter((n: Note) => !n.resolved);

  return (
    <div className="space-y-1">
      <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
        노트 ({active.length})
      </span>
      {active.length === 0 && (
        <p className="text-xs text-slate-400">노트 없음</p>
      )}
      {active.map((note: Note) => (
        <div key={note.id}>
          <button
            className={`w-full flex items-start gap-1.5 text-left rounded px-2 py-1.5 hover:opacity-80 ${NOTE_COLOR[note.note_type]}`}
            onClick={() => setExpanded(expanded === note.id ? null : note.id)}
          >
            <span className="shrink-0">{NOTE_ICON[note.note_type]}</span>
            <span className="text-xs">{note.summary}</span>
          </button>
          {expanded === note.id && <NoteDetail id={note.id} />}
        </div>
      ))}
    </div>
  );
}
