export function EpicChip({ title }: { title: string }) {
  return (
    <span className="text-xs bg-indigo-100 text-indigo-700 rounded px-1.5 py-0.5 truncate max-w-[100px]">
      {title}
    </span>
  );
}
