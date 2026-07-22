import React from 'react';
import { toast } from 'sonner';

interface Props {
  type: 'issue' | 'epic' | 'mission';
  id: number;
  prefix?: string;
  className?: string;
  showIcon?: boolean;
}

export function CopyableId({ type, id, prefix = '#', className = '', showIcon = true }: Props) {
  const formattedId = `[engram ${type}-#${id}]`;

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(formattedId);
    toast.success(`"${formattedId}" 클립보드에 복사되었습니다`);
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={`클릭하여 "${formattedId}" 복사`}
      className={`inline-flex items-center gap-1 group font-mono hover:text-indigo-600 transition-colors cursor-pointer ${className}`}
    >
      <span>{prefix}{id}</span>
      {showIcon && (
        <span className="text-[10px] opacity-40 group-hover:opacity-100 transition-opacity">
          📋
        </span>
      )}
    </button>
  );
}
