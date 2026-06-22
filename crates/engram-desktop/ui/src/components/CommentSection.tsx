import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { noteList, noteAdd } from '../ipc/invoke';
import { Markdown } from './Markdown';
import type { Note } from '../ipc/types';

interface Props {
  issueId: number;
}

export function CommentSection({ issueId }: Props) {
  const qc = useQueryClient();
  const [text, setText] = useState('');

  const { data: notes = [] } = useQuery({
    queryKey: ['notes', issueId],
    queryFn: () => noteList(issueId),
  });

  const comments = notes.filter((n: Note) => n.note_type === 'comment');

  const addComment = useMutation({
    mutationFn: (summary: string) =>
      noteAdd({
        issue_id: issueId,
        note_type: 'comment',
        summary,
        author: 'user',
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notes', issueId] });
      qc.invalidateQueries({ queryKey: ['issue', issueId] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['history', 'issue', issueId] });
      setText('');
    },
    onError: (err) => toast.error(`코멘트 추가 실패: ${err}`),
  });

  function submit() {
    const v = text.trim();
    if (v.length === 0) return;
    addComment.mutate(v);
  }

  return (
    <section>
      <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
        코멘트 ({comments.length})
      </h3>

      <div className="space-y-1 mb-2">
        {comments.length === 0 && (
          <p className="text-xs text-slate-400">코멘트 없음</p>
        )}
        {comments.map((c: Note) => (
          <div key={c.id} className="px-2 py-1.5 bg-slate-50 rounded border border-slate-200">
            <div className="flex items-center justify-between mb-0.5">
              <span className="text-xs font-medium text-slate-600">{c.author}</span>
              <span className="text-xs text-slate-400">{c.created_at.slice(0, 16).replace('T', ' ')}</span>
            </div>
            <Markdown>{c.summary}</Markdown>
          </div>
        ))}
      </div>

      <div className="flex gap-2">
        <input
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submit(); } }}
          placeholder="코멘트 추가…"
          className="flex-1 px-2 py-1 text-sm border border-slate-200 rounded focus:outline-none focus:border-indigo-400"
        />
        <button
          type="button"
          onClick={submit}
          disabled={text.trim().length === 0 || addComment.isPending}
          className="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded disabled:opacity-50"
        >
          추가
        </button>
      </div>
    </section>
  );
}
