import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueLinks, issueLink, issueUnlink } from '../ipc/invoke';
import type { IssueLink, LinkType } from '../ipc/types';

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
}

export function IssueLinkSection({ issueId }: Props) {
  const qc = useQueryClient();
  const [showAdd, setShowAdd] = useState(false);
  const [targetId, setTargetId] = useState('');
  const [linkType, setLinkType] = useState<LinkType>('blocks');

  const { data: links = [] } = useQuery({
    queryKey: ['issueLinks', issueId],
    queryFn: () => issueLinks(issueId),
  });

  const addLink = useMutation({
    mutationFn: () => issueLink(issueId, Number(targetId), linkType),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueLinks', issueId] });
      qc.invalidateQueries({ queryKey: ['blockingGraph'] });
      toast.success('이슈가 연결되었습니다');
      setTargetId('');
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
          + 이슈 연결
        </button>
      </div>

      {showAdd && (
        <div className="flex gap-2 mb-2 p-2 bg-slate-50 rounded border border-slate-200">
          <select
            value={linkType}
            onChange={(e) => setLinkType(e.target.value as LinkType)}
            className="px-2 py-1 text-xs border border-slate-300 rounded bg-white"
          >
            <option value="blocks">차단함 (blocks)</option>
            <option value="relates_to">관계 (relates_to)</option>
            <option value="duplicates">중복 (duplicates)</option>
          </select>
          <input
            type="number"
            value={targetId}
            onChange={(e) => setTargetId(e.target.value)}
            placeholder="이슈 #ID"
            className="w-24 px-2 py-1 text-xs border border-slate-300 rounded"
          />
          <button
            type="button"
            disabled={!targetId || addLink.isPending}
            onClick={() => addLink.mutate()}
            className="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded disabled:opacity-50"
          >
            연결
          </button>
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
              onRemove={() => removeLink.mutate(link.id)}
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
              onRemove={() => removeLink.mutate(link.id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function LinkRow({
  link, direction, onRemove,
}: {
  link: IssueLink;
  direction: 'outgoing' | 'incoming';
  onRemove: () => void;
}) {
  const otherId = direction === 'outgoing' ? link.target_id : link.source_id;
  const arrow = direction === 'outgoing' ? '→' : '←';
  return (
    <div className={`flex items-center justify-between text-xs px-2 py-1 border rounded ${LINK_TYPE_COLOR[link.link_type]}`}>
      <span>
        {arrow} <span className="font-semibold">{LINK_TYPE_LABEL[link.link_type]}</span> #{otherId}
      </span>
      <button
        type="button"
        onClick={onRemove}
        className="text-slate-400 hover:text-red-600 ml-2 leading-none"
        title="연결 해제"
      >
        ×
      </button>
    </div>
  );
}
