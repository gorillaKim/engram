import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface Props {
  children: string;
  className?: string;
}

export function Markdown({ children, className = '' }: Props) {
  return (
    <div className={`break-words ${className}`}>
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        h1: ({ children }) => <h1 className="text-base font-bold text-slate-800 mb-1">{children}</h1>,
        h2: ({ children }) => <h2 className="text-sm font-bold text-slate-700 mb-1">{children}</h2>,
        h3: ({ children }) => <h3 className="text-sm font-semibold text-slate-700 mb-1">{children}</h3>,
        p: ({ children }) => <p className="text-sm text-slate-600 leading-relaxed mb-2 last:mb-0">{children}</p>,
        ul: ({ children }) => <ul className="text-sm text-slate-600 list-disc list-inside space-y-0.5 mb-2">{children}</ul>,
        ol: ({ children }) => <ol className="text-sm text-slate-600 list-decimal list-inside space-y-0.5 mb-2">{children}</ol>,
        li: ({ children }) => <li className="leading-relaxed">{children}</li>,
        code: ({ children, className }) => {
          const isBlock = className?.includes('language-');
          return isBlock
            ? <code className="block bg-slate-100 rounded px-3 py-2 text-xs font-mono text-slate-700 overflow-x-auto whitespace-pre">{children}</code>
            : <code className="bg-slate-100 rounded px-1 py-0.5 text-xs font-mono text-indigo-700 break-all">{children}</code>;
        },
        pre: ({ children }) => <pre className="bg-slate-100 rounded p-3 mb-2 overflow-x-auto">{children}</pre>,
        blockquote: ({ children }) => <blockquote className="border-l-2 border-slate-300 pl-3 italic text-slate-500 mb-2">{children}</blockquote>,
        a: ({ href, children }) => <a href={href} target="_blank" rel="noreferrer" className="text-indigo-600 hover:underline break-all">{children}</a>,
        strong: ({ children }) => <strong className="font-semibold text-slate-800">{children}</strong>,
        em: ({ children }) => <em className="italic text-slate-600">{children}</em>,
        hr: () => <hr className="border-slate-200 my-2" />,
        table: ({ children }) => (
          <div className="overflow-x-auto w-full mb-2 border border-slate-200 rounded">
            <table className="text-xs min-w-full border-collapse">{children}</table>
          </div>
        ),
        th: ({ children }) => <th className="text-left px-2 py-1 bg-slate-50 border-b border-r last:border-r-0 border-slate-200 font-semibold text-slate-700 whitespace-nowrap">{children}</th>,
        td: ({ children }) => <td className="px-2 py-1 border-b border-r last:border-r-0 border-slate-200 text-slate-600 whitespace-nowrap">{children}</td>,
      }}
    >
      {children}
    </ReactMarkdown>
    </div>
  );
}
