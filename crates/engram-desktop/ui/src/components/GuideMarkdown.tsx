import { useEffect, useRef, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import mermaid from 'mermaid';

// Mermaid 초기화
mermaid.initialize({
  startOnLoad: false,
  theme: 'neutral',
  securityLevel: 'loose',
  themeVariables: {
    background: '#f8fafc',
    primaryColor: '#e0e7ff',
    primaryTextColor: '#312e81',
    lineColor: '#64748b',
  }
});

function Mermaid({ chart }: { chart: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const [svg, setSvg] = useState<string>('');
  const [error, setError] = useState<boolean>(false);

  useEffect(() => {
    let isMounted = true;
    const id = `mermaid-${Math.random().toString(36).substring(2, 9)}`;

    const renderChart = async () => {
      try {
        // 기존 렌더링 결과 지우기
        if (ref.current) {
          ref.current.innerHTML = '';
        }
        
        const cleanChart = chart.trim();
        const { svg: svgHtml } = await mermaid.render(id, cleanChart);
        
        if (isMounted) {
          setSvg(svgHtml);
          setError(false);
        }
      } catch (err) {
        console.error('Mermaid rendering failed:', err);
        if (isMounted) {
          setError(true);
        }
        // mermaid 에러 엘리먼트가 DOM에 남아 레이아웃을 해치는 것 방지
        const errEl = document.getElementById(`d${id}`);
        if (errEl) {
          errEl.remove();
        }
      }
    };

    renderChart();

    return () => {
      isMounted = false;
    };
  }, [chart]);

  if (error) {
    return (
      <pre className="text-xs bg-rose-50 text-rose-600 p-3 rounded-lg border border-rose-200 overflow-x-auto font-mono my-4">
        {chart}
      </pre>
    );
  }

  return (
    <div 
      ref={ref} 
      className="flex justify-center my-6 p-6 bg-slate-50 border border-slate-200/60 rounded-xl overflow-x-auto shadow-sm"
      dangerouslySetInnerHTML={{ __html: svg }} 
    />
  );
}

interface Props {
  children: string;
  className?: string;
}

export function GuideMarkdown({ children, className = '' }: Props) {
  const formattedChildren = children.replace(/\\n/g, '\n');

  return (
    <div className={`break-words leading-relaxed text-slate-700 ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => (
            <h1 className="text-2xl font-bold text-slate-900 mt-8 mb-4 border-b border-slate-200 pb-2 flex items-center gap-2">
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2 className="text-xl font-bold text-slate-800 mt-6 mb-3 flex items-center gap-2">
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3 className="text-lg font-semibold text-slate-800 mt-4 mb-2">
              {children}
            </h3>
          ),
          p: ({ children }) => (
            <p className="text-slate-600 mb-4 text-[15px] leading-relaxed last:mb-0">
              {children}
            </p>
          ),
          ul: ({ children }) => (
            <ul className="list-disc list-outside pl-6 space-y-1.5 mb-4 text-slate-600 text-[15px]">
              {children}
            </ul>
          ),
          ol: ({ children }) => (
            <ol className="list-decimal list-outside pl-6 space-y-1.5 mb-4 text-slate-600 text-[15px]">
              {children}
            </ol>
          ),
          li: ({ children }) => <li className="pl-0.5">{children}</li>,
          code: ({ children, className }) => {
            const isBlock = className?.includes('language-');
            const lang = className?.replace('language-', '') || '';
            const codeString = String(children).replace(/\n$/, '');

            if (isBlock && lang === 'mermaid') {
              return <Mermaid chart={codeString} />;
            }

            return isBlock ? (
              <pre className="bg-slate-900 rounded-xl p-4 text-xs font-mono text-slate-100 overflow-x-auto whitespace-pre my-4 shadow-inner">
                <code className="font-mono">{codeString}</code>
              </pre>
            ) : (
              <code className="bg-slate-100 border border-slate-200/50 rounded-md px-1.5 py-0.5 text-xs font-mono text-indigo-600 font-semibold mx-0.5 break-all">
                {children}
              </code>
            );
          },
          pre: ({ children }) => <>{children}</>,
          blockquote: ({ children }) => {
            // Children 안에서 텍스트 노드를 파싱하여 GitHub style alerts인지 감지합니다.
            const contentArray = Array.isArray(children) ? children : [children];
            
            // p 태그 내부의 텍스트가 [!NOTE], [!IMPORTANT], [!WARNING], [!TIP] 등으로 시작하는지 검사
            let alertType: 'note' | 'important' | 'warning' | 'tip' | null = null;
            let rawText = '';
            
            // children 리스트에서 텍스트 내용을 추출
            const extractText = (node: any): string => {
              if (!node) return '';
              if (typeof node === 'string') return node;
              if (node.props && node.props.children) {
                if (Array.isArray(node.props.children)) {
                  return node.props.children.map(extractText).join('');
                }
                return extractText(node.props.children);
              }
              return '';
            };

            const fullText = contentArray.map(extractText).join('').trim();

            if (fullText.startsWith('[!NOTE]')) {
              alertType = 'note';
              rawText = fullText.replace('[!NOTE]', '').trim();
            } else if (fullText.startsWith('[!IMPORTANT]')) {
              alertType = 'important';
              rawText = fullText.replace('[!IMPORTANT]', '').trim();
            } else if (fullText.startsWith('[!WARNING]')) {
              alertType = 'warning';
              rawText = fullText.replace('[!WARNING]', '').trim();
            } else if (fullText.startsWith('[!TIP]')) {
              alertType = 'tip';
              rawText = fullText.replace('[!TIP]', '').trim();
            }

            if (alertType) {
              const config = {
                note: {
                  bg: 'bg-blue-50/70 border-blue-400 text-blue-800',
                  title: '알림',
                  icon: (
                    <svg className="w-5 h-5 text-blue-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  ),
                },
                important: {
                  bg: 'bg-indigo-50/70 border-indigo-400 text-indigo-900',
                  title: '중요',
                  icon: (
                    <svg className="w-5 h-5 text-indigo-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                  ),
                },
                warning: {
                  bg: 'bg-amber-50/70 border-amber-400 text-amber-900',
                  title: '주의',
                  icon: (
                    <svg className="w-5 h-5 text-amber-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                  ),
                },
                tip: {
                  bg: 'bg-emerald-50/70 border-emerald-400 text-emerald-900',
                  title: '팁',
                  icon: (
                    <svg className="w-5 h-5 text-emerald-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                    </svg>
                  ),
                },
              }[alertType];

              return (
                <div className={`border-l-4 p-4 rounded-r-xl my-4 flex gap-3 ${config.bg}`}>
                  {config.icon}
                  <div className="flex-1">
                    <div className="font-bold text-[14px] mb-1">{config.title}</div>
                    <p className="text-[14px] leading-relaxed">{rawText}</p>
                  </div>
                </div>
              );
            }

            return (
              <blockquote className="border-l-4 border-slate-300 pl-4 italic text-slate-500 my-4 bg-slate-50/50 py-1 pr-2 rounded-r-lg">
                {children}
              </blockquote>
            );
          },
          a: ({ href, children }) => (
            <a
              href={href}
              target="_blank"
              rel="noreferrer"
              className="text-indigo-600 font-medium hover:underline hover:text-indigo-500 break-all transition-colors"
            >
              {children}
            </a>
          ),
          strong: ({ children }) => (
            <strong className="font-semibold text-slate-900">{children}</strong>
          ),
          em: ({ children }) => (
            <em className="italic text-slate-600">{children}</em>
          ),
          hr: () => <hr className="border-slate-200 my-6" />,
          table: ({ children }) => (
            <div className="overflow-x-auto w-full my-6 border border-slate-200/80 rounded-xl shadow-sm">
              <table className="text-sm min-w-full border-collapse">{children}</table>
            </div>
          ),
          th: ({ children }) => (
            <th className="text-left px-4 py-2.5 bg-slate-50 border-b border-r last:border-r-0 border-slate-200 font-semibold text-slate-800 whitespace-nowrap">
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td className="px-4 py-2.5 border-b border-r last:border-r-0 border-slate-200 text-slate-600 text-[14px] bg-white">
              {children}
            </td>
          ),
        }}
      >
        {formattedChildren}
      </ReactMarkdown>
    </div>
  );
}
