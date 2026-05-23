import { useState, useEffect, useRef, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueLinks, issueLink, issueUnlink, issueList } from '../ipc/invoke';
import type { Issue, IssueLink, LinkType } from '../ipc/types';
import { useUIStore } from '../store/ui';

const LINK_TYPE_LABEL: Record<LinkType, string> = {
  blocks: '차단함',
  relates_to: '관계',
  duplicates: '중복',
};

const LINK_TYPE_COLOR: Record<LinkType, string> = {
  blocks: 'bg-red-50 text-red-700 border-red-200',
  relates_to: 'bg-blue-50 text-blue-700 border-blue-200',
  duplicates: 'bg-slate-100 text-slate-700 border-slate-300',
};

interface Props {
  issueId: number;
  /** 연결된 이슈를 IssueDetail 패널에 열 때 selectProject 호출에 사용. */
  projectKey?: string;
}

export function IssueLinkSection({ issueId, projectKey }: Props) {
  const qc = useQueryClient();
  const { selectIssue, selectProject } = useUIStore();
  const [showAdd, setShowAdd] = useState(false);
  const [targetId, setTargetId] = useState<number | null>(null);
  const [linkType, setLinkType] = useState<LinkType>('blocks');

  const { data: links = [] } = useQuery({
    queryKey: ['issueLinks', issueId],
    queryFn: () => issueLinks(issueId),
  });

  // 검색용 + 제목 표시용 — 모든 이슈 (cross-project 연결 허용).
  // enabled 조건 없이 항상 fetch → issueMap 이 항상 채워져 제목 표시 가능.
  const { data: allIssues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'all-for-link'],
    queryFn: () => issueList({}),
    staleTime: 30_000,
  });

  const addLink = useMutation({
    mutationFn: () => {
      if (!targetId) throw new Error('대상 이슈를 선택하세요');
      return issueLink(issueId, targetId, linkType);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueLinks', issueId] });
      qc.invalidateQueries({ queryKey: ['blockingGraph'] });
      toast.success('이슈가 연결되었습니다');
      setTargetId(null);
      setShowAdd(false);
    },
    onError: (err) => toast.error(`연결 실패: ${err}`),
  });

  const removeLink = useMutation({
    mutationFn: (link_id: number) => issueUnlink(link_id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueLinks', issueId] });
      qc.invalidateQueries({ queryKey: ['blockingGraph'] });
    },
    onError: (err) => toast.error(`연결 해제 실패: ${err}`),
  });

  const outgoing = links.filter((l: IssueLink) => l.source_id === issueId);
  const incoming = links.filter((l: IssueLink) => l.target_id === issueId);

  // 자기 자신과 이미 연결된 이슈는 후보에서 제외
  const linkedIds = useMemo(() => {
    const s = new Set<number>([issueId]);
    for (const l of links) { s.add(l.source_id); s.add(l.target_id); }
    return s;
  }, [issueId, links]);

  const candidates = useMemo(
    () => allIssues.filter((i) => !linkedIds.has(i.id)),
    [allIssues, linkedIds],
  );

  // 제목/ID 빠른 lookup — LinkRow 가 사용
  const issueMap = useMemo(() => {
    const m = new Map<number, Issue>();
    for (const i of allIssues) m.set(i.id, i);
    return m;
  }, [allIssues]);

  return (
    <section>
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
          이슈 연결 ({links.length})
        </h3>
        <button
          type="button"
          onClick={() => setShowAdd((v) => !v)}
          className="text-xs px-2 py-0.5 text-indigo-600 hover:bg-indigo-50 rounded"
        >
          {showAdd ? '취소' : '+ 이슈 연결'}
        </button>
      </div>

      {showAdd && (
        <div className="mb-2 p-2 bg-slate-50 rounded border border-slate-200 space-y-2">
          <select
            value={linkType}
            onChange={(e) => setLinkType(e.target.value as LinkType)}
            className="w-full px-2 py-1 text-xs border border-slate-300 rounded bg-white"
          >
            <option value="blocks">차단함 (blocks)</option>
            <option value="relates_to">관계 (relates_to)</option>
            <option value="duplicates">중복 (duplicates)</option>
          </select>

          <IssueSearchSelect
            candidates={candidates}
            value={targetId}
            onChange={setTargetId}
          />

          <div className="flex justify-end">
            <button
              type="button"
              disabled={!targetId || addLink.isPending}
              onClick={() => addLink.mutate()}
              className="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded disabled:opacity-50"
            >
              {addLink.isPending ? '연결 중…' : '연결'}
            </button>
          </div>
        </div>
      )}

      {links.length === 0 && !showAdd && (
        <p className="text-xs text-slate-400">연결된 이슈 없음</p>
      )}

      {outgoing.length > 0 && (
        <div className="space-y-1 mb-2">
          {outgoing.map((link: IssueLink) => (
            <LinkRow
              key={link.id}
              link={link}
              direction="outgoing"
              other={issueMap.get(link.target_id)}
              onRemove={() => removeLink.mutate(link.id)}
              onOpen={() => {
                selectProject(projectKey ?? null);
                selectIssue(link.target_id);
              }}
            />
          ))}
        </div>
      )}

      {incoming.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs text-slate-500">받은 연결</span>
          {incoming.map((link: IssueLink) => (
            <LinkRow
              key={link.id}
              link={link}
              direction="incoming"
              other={issueMap.get(link.source_id)}
              onRemove={() => removeLink.mutate(link.id)}
              onOpen={() => {
                selectProject(projectKey ?? null);
                selectIssue(link.source_id);
              }}
            />
          ))}
        </div>
      )}
    </section>
  );
}

// ── Searchable combobox (이름/ID 로 검색해서 이슈 선택) ─────────────────────────

function IssueSearchSelect({
  candidates,
  value,
  onChange,
}: {
  candidates: Issue[];
  value: number | null;
  onChange: (id: number | null) => void;
}) {
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState(false);
  const [highlight, setHighlight] = useState(0);
  const wrapRef = useRef<HTMLDivElement>(null);

  const selected = useMemo(
    () => candidates.find((c) => c.id === value),
    [candidates, value],
  );

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return candidates.slice(0, 30);
    const idMatch = candidates.filter(
      (c) => `${c.id}` === q || `#${c.id}` === q,
    );
    const titleMatch = candidates.filter(
      (c) => c.title.toLowerCase().includes(q) && !idMatch.includes(c),
    );
    return [...idMatch, ...titleMatch].slice(0, 30);
  }, [candidates, query]);

  // 입력창 변경되면 highlight 리셋
  useEffect(() => { setHighlight(0); }, [query]);

  // 바깥 클릭 시 닫기
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (wrapRef.current && !wrapRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  function pick(issue: Issue) {
    onChange(issue.id);
    setQuery('');
    setOpen(false);
  }

  function clear() {
    onChange(null);
    setQuery('');
  }

  return (
    <div ref={wrapRef} className="relative">
      {selected ? (
        <div className="flex items-center gap-2 px-2 py-1 border border-slate-300 rounded bg-white">
          <span className="text-xs text-slate-500">#{selected.id}</span>
          <span className="text-xs text-slate-800 flex-1 truncate">{selected.title}</span>
          <button
            type="button"
            onClick={clear}
            className="text-xs text-slate-400 hover:text-red-600"
          >
            ✕
          </button>
        </div>
      ) : (
        <input
          type="text"
          value={query}
          onChange={(e) => { setQuery(e.target.value); setOpen(true); }}
          onFocus={() => setOpen(true)}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') {
              e.preventDefault();
              setHighlight((h) => Math.min(h + 1, filtered.length - 1));
            } else if (e.key === 'ArrowUp') {
              e.preventDefault();
              setHighlight((h) => Math.max(h - 1, 0));
            } else if (e.key === 'Enter') {
              e.preventDefault();
              if (filtered[highlight]) pick(filtered[highlight]);
            } else if (e.key === 'Escape') {
              setOpen(false);
            }
          }}
          placeholder="이슈 제목 또는 #ID 로 검색…"
          className="w-full px-2 py-1 text-xs border border-slate-300 rounded bg-white focus:outline-none focus:border-indigo-400"
        />
      )}

      {open && !selected && filtered.length > 0 && (
        <ul className="absolute top-full left-0 right-0 z-10 mt-1 max-h-60 overflow-y-auto bg-white border border-slate-200 rounded shadow-lg">
          {filtered.map((c, idx) => (
            <li
              key={c.id}
              onMouseDown={(e) => { e.preventDefault(); pick(c); }}
              onMouseEnter={() => setHighlight(idx)}
              className={`px-2 py-1.5 text-xs cursor-pointer flex items-baseline gap-2 ${
                idx === highlight ? 'bg-indigo-50' : 'hover:bg-slate-50'
              }`}
            >
              <span className="text-slate-400 shrink-0">#{c.id}</span>
              <span className="text-slate-800 flex-1 truncate">{c.title}</span>
              <span className="text-slate-400 text-[10px] shrink-0">{c.status}</span>
            </li>
          ))}
        </ul>
      )}

      {open && !selected && filtered.length === 0 && query.trim().length > 0 && (
        <p className="absolute top-full left-0 mt-1 text-xs text-slate-400 px-2 py-1 bg-white border border-slate-200 rounded shadow">
          검색 결과 없음
        </p>
      )}
    </div>
  );
}

// ── Link row ────────────────────────────────────────────────────────────────

function LinkRow({
  link, direction, other, onRemove, onOpen,
}: {
  link: IssueLink;
  direction: 'outgoing' | 'incoming';
  other?: Issue;
  onRemove: () => void;
  onOpen: () => void;
}) {
  const otherId = direction === 'outgoing' ? link.target_id : link.source_id;
  const arrow = direction === 'outgoing' ? '→' : '←';
  return (
    <div className={`flex items-center justify-between text-xs px-2 py-1 border rounded ${LINK_TYPE_COLOR[link.link_type]}`}>
      <span className="flex items-baseline gap-1.5 min-w-0 flex-1 overflow-hidden">
        <span className="shrink-0">{arrow}</span>
        <span className="font-semibold shrink-0">{LINK_TYPE_LABEL[link.link_type]}</span>
        <span className="shrink-0">#{otherId}</span>
        {other && <span className="truncate opacity-75">{other.title}</span>}
      </span>
      <span className="flex items-center gap-1 ml-2 shrink-0">
        <button
          type="button"
          onClick={onOpen}
          className="text-slate-400 hover:text-indigo-600 leading-none px-0.5"
          title="이 이슈 열기"
        >
          ↗
        </button>
        <button
          type="button"
          onClick={onRemove}
          className="text-slate-400 hover:text-red-600 leading-none"
          title="연결 해제"
        >
          ×
        </button>
      </span>
    </div>
  );
}
