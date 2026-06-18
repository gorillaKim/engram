import { useState } from 'react';
import { ChevronDown, HelpCircle, BookOpen, Search, Info } from 'lucide-react';
import { GuideMarkdown } from '../components/GuideMarkdown';

// 마크다운 가이드 파일 로드 (?raw)
import introMd from '../assets/guides/intro.md?raw';
import issueManagementMd from '../assets/guides/issue-management.md?raw';
import sprintOperationMd from '../assets/guides/sprint-operation.md?raw';
import mcpIntegrationMd from '../assets/guides/mcp-integration.md?raw';

// FAQ 데이터 로드
import faqData from '../assets/guides/faq.json';

type MainTab = 'guide' | 'faq';

interface GuideChapter {
  id: string;
  title: string;
  icon: string;
  content: string;
}

const GUIDE_CHAPTERS: GuideChapter[] = [
  {
    id: 'intro',
    title: '🚀 소개 및 기본 개념',
    icon: '🚀',
    content: introMd,
  },
  {
    id: 'issues',
    title: '📋 이슈 및 태스크 관리',
    icon: '📋',
    content: issueManagementMd,
  },
  {
    id: 'sprints',
    title: '🔄 스프린트 및 회고',
    icon: '🔄',
    content: sprintOperationMd,
  },
  {
    id: 'mcp',
    title: '🔌 MCP 및 에이전트 연동',
    icon: '🔌',
    content: mcpIntegrationMd,
  },
];

const CATEGORY_MAP: Record<string, string> = {
  concept: '개념 및 설계',
  workflow: '업무 프로세스',
  mcp: 'AI 에이전트 연동',
  cli: 'CLI / 기타',
};

export function Guide() {
  const [mainTab, setMainTab] = useState<MainTab>('guide');
  const [selectedChapterId, setSelectedChapterId] = useState<string>('intro');
  const [faqCategory, setFaqCategory] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [openFaqIds, setOpenFaqIds] = useState<number[]>([]);

  // 아코디언 토글
  const toggleFaq = (id: number) => {
    setOpenFaqIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  };

  const selectedChapter = GUIDE_CHAPTERS.find((ch) => ch.id === selectedChapterId) || GUIDE_CHAPTERS[0];

  // FAQ 필터링 및 검색
  const filteredFaqs = faqData.filter((faq) => {
    const matchesCategory = faqCategory === 'all' || faq.category === faqCategory;
    const matchesSearch =
      faq.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
      faq.answer.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  return (
    <div className="flex h-full bg-slate-50/50 overflow-hidden">
      {/* 1. 좌측 사이드바 */}
      <aside className="w-80 border-r border-slate-200/80 bg-white flex flex-col flex-shrink-0">
        {/* 상단 메인 탭 전환 */}
        <div className="p-4 border-b border-slate-100">
          <div className="flex p-1 bg-slate-100 rounded-xl">
            <button
              onClick={() => setMainTab('guide')}
              className={`flex-1 flex items-center justify-center gap-2 py-2 text-xs font-semibold rounded-lg transition-all ${
                mainTab === 'guide'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              <BookOpen className="w-4 h-4" />
              사용 가이드
            </button>
            <button
              onClick={() => setMainTab('faq')}
              className={`flex-1 flex items-center justify-center gap-2 py-2 text-xs font-semibold rounded-lg transition-all ${
                mainTab === 'faq'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              <HelpCircle className="w-4 h-4" />
              자주 묻는 질문
            </button>
          </div>
        </div>

        {/* 탭 내용에 따른 사이드바 리스트 */}
        <div className="flex-1 overflow-y-auto p-3 space-y-1">
          {mainTab === 'guide' ? (
            // 사용 가이드 목차 목록
            GUIDE_CHAPTERS.map((ch) => (
              <button
                key={ch.id}
                onClick={() => setSelectedChapterId(ch.id)}
                className={`w-full text-left px-4 py-3 rounded-xl transition-all flex items-center gap-3 text-[14px] ${
                  selectedChapterId === ch.id
                    ? 'bg-indigo-50/80 text-indigo-600 font-semibold border border-indigo-100/55'
                    : 'text-slate-600 hover:bg-slate-50 hover:text-slate-800'
                }`}
              >
                <span className="text-base">{ch.title.split(' ')[0]}</span>
                <span className="flex-1 truncate">{ch.title.substring(ch.title.indexOf(' ') + 1)}</span>
              </button>
            ))
          ) : (
            // Q&A 카테고리 필터
            <div className="space-y-4">
              {/* FAQ 검색창 */}
              <div className="px-2 pt-2">
                <div className="relative">
                  <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Q&A 검색…"
                    className="w-full bg-slate-100 border-none rounded-xl pl-9 pr-4 py-2 text-xs focus:ring-2 focus:ring-indigo-500/20 text-slate-700 placeholder-slate-400"
                  />
                </div>
              </div>

              {/* 카테고리 버튼 그룹 */}
              <div className="space-y-1">
                <div className="px-3 text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-2">
                  카테고리
                </div>
                {[
                  { key: 'all', label: '전체 보기', count: faqData.length },
                  { key: 'concept', label: '개념 및 설계', count: faqData.filter(f => f.category === 'concept').length },
                  { key: 'workflow', label: '업무 프로세스', count: faqData.filter(f => f.category === 'workflow').length },
                  { key: 'mcp', label: 'AI 에이전트 연동', count: faqData.filter(f => f.category === 'mcp').length },
                  { key: 'cli', label: 'CLI / 기타', count: faqData.filter(f => f.category === 'cli').length },
                ].map((cat) => (
                  <button
                    key={cat.key}
                    onClick={() => setFaqCategory(cat.key)}
                    className={`w-full text-left px-4 py-2.5 rounded-xl transition-all flex items-center justify-between text-xs ${
                      faqCategory === cat.key
                        ? 'bg-indigo-50/80 text-indigo-600 font-semibold border border-indigo-100/55'
                        : 'text-slate-600 hover:bg-slate-50 hover:text-slate-800'
                    }`}
                  >
                    <span>{cat.label}</span>
                    <span className={`px-2 py-0.5 rounded-full text-[10px] ${
                      faqCategory === cat.key ? 'bg-indigo-100 text-indigo-700' : 'bg-slate-100 text-slate-500'
                    }`}>
                      {cat.count}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* 사이드바 하단 정보 영역 */}
        <div className="p-4 bg-slate-50/60 border-t border-slate-100 flex items-center gap-3">
          <Info className="w-5 h-5 text-indigo-500 flex-shrink-0" />
          <div className="text-[11px] text-slate-500 leading-normal">
            도움이 더 필요하신가요? 터미널에서 <code className="bg-slate-200/80 px-1 rounded font-mono font-bold text-slate-700">engram --help</code> 명령어를 입력해 보세요.
          </div>
        </div>
      </aside>

      {/* 2. 우측 상세 본문 */}
      <main className="flex-1 overflow-y-auto bg-white p-8 md:p-12">
        <div className="max-w-3xl mx-auto">
          {mainTab === 'guide' ? (
            // 사용 가이드 렌더링
            <article className="animate-fade-in">
              <GuideMarkdown>{selectedChapter.content}</GuideMarkdown>
            </article>
          ) : (
            // Q&A 아코디언 리스트
            <div className="space-y-4">
              <div className="mb-6">
                <h1 className="text-2xl font-bold text-slate-900 mb-2">자주 묻는 질문 (Q&A)</h1>
                <p className="text-sm text-slate-500">Engram 사용 및 연동 중에 자주 겪는 상황에 대한 해결책입니다.</p>
              </div>

              {filteredFaqs.length > 0 ? (
                filteredFaqs.map((faq) => {
                  const isOpen = openFaqIds.includes(faq.id);
                  return (
                    <div
                      key={faq.id}
                      className={`border border-slate-200/60 rounded-xl overflow-hidden bg-white shadow-sm transition-all duration-300 ${
                        isOpen ? 'ring-1 ring-indigo-500/25 border-indigo-200' : 'hover:border-slate-300/80'
                      }`}
                    >
                      {/* 질문 버튼 */}
                      <button
                        onClick={() => toggleFaq(faq.id)}
                        className="w-full flex items-center justify-between p-4 md:p-5 text-left transition-colors"
                      >
                        <div className="flex items-start gap-3.5">
                          <span className="inline-flex items-center justify-center w-6 h-6 rounded-lg bg-indigo-50 text-indigo-600 font-bold text-xs mt-0.5">
                            Q
                          </span>
                          <div className="flex flex-col gap-1">
                            <span className="text-[10px] font-semibold text-indigo-500 uppercase tracking-wider">
                              {CATEGORY_MAP[faq.category]}
                            </span>
                            <span className="text-sm md:text-base font-semibold text-slate-800 leading-snug">
                              {faq.question}
                            </span>
                          </div>
                        </div>
                        <ChevronDown
                          className={`w-5 h-5 text-slate-400 flex-shrink-0 transition-transform duration-300 ${
                            isOpen ? 'rotate-180 text-indigo-500' : ''
                          }`}
                        />
                      </button>

                      {/* 답변 영역 (아코디언 애니메이션) */}
                      <div
                        className={`transition-all duration-300 ease-in-out overflow-hidden border-slate-100 ${
                          isOpen ? 'max-h-[500px] border-t opacity-100' : 'max-h-0 opacity-0 pointer-events-none'
                        }`}
                      >
                        <div className="p-5 bg-slate-50/50 flex items-start gap-3.5">
                          <span className="inline-flex items-center justify-center w-6 h-6 rounded-lg bg-emerald-50 text-emerald-600 font-bold text-xs mt-0.5 flex-shrink-0">
                            A
                          </span>
                          <div className="flex-1 text-sm text-slate-600 leading-relaxed">
                            <GuideMarkdown>{faq.answer}</GuideMarkdown>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })
              ) : (
                <div className="text-center py-16 border border-dashed border-slate-200 rounded-xl bg-slate-50/30">
                  <HelpCircle className="w-10 h-10 text-slate-300 mx-auto mb-3" />
                  <p className="text-sm font-medium text-slate-500">검색 결과에 맞는 질문이 없습니다.</p>
                  <p className="text-xs text-slate-400 mt-1">다른 키워드를 입력해 보세요.</p>
                </div>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
