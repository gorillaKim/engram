/**
 * ADR-0015에 근거한 SSE 및 미확정 프로토콜 계약의 단일 정정 지점(Single Source of Truth).
 * 621 transport 등 후속 SSE 연동 모듈은 본 파일에 정의된 스펙과 Mock 데이터만을 참조하여 구현해야 하며,
 * 실서버 SSE 라이브 적용 시 본 파일만 수정하여 연동 사양 변경을 일원화합니다.
 */

/**
 * SSE 이벤트 메시지 타입 정의
 */
export type SSEEventRole = 'system' | 'user' | 'assistant' | 'tool';

export interface SSEMessageDelta {
  role?: SSEEventRole;
  content?: string;
  tool_calls?: Array<{
    id: string;
    type: 'function';
    function: {
      name: string;
      arguments: string; // JSON String
    };
  }>;
}

export interface SSEEventDataMap {
  /**
   * metadata: 대화 ID 및 세션 초기 메타데이터 전송
   */
  metadata: {
    conversation_id: string;
    created_at: string;
  };
  
  /**
   * status: 에이전트의 현재 실행 단계/상태 알림
   */
  status: {
    state: 'thinking' | 'searching' | 'responding' | 'idle';
    message?: string;
  };

  /**
   * delta: 스트리밍 텍스트 조각 및 도구 호출 정보
   */
  delta: SSEMessageDelta;

  /**
   * tool_result: 도구 실행 결과 피드백
   */
  tool_result: {
    tool_call_id: string;
    output: string;
  };

  /**
   * done: 스트리밍 종료 및 최종 요약 데이터
   */
  done: {
    conversation_id: string;
    total_tokens?: number;
    finish_reason: 'stop' | 'length' | 'tool_calls';
  };
}

export type SSEEventType = keyof SSEEventDataMap;

export interface SSEEventFrame<T extends SSEEventType> {
  event: T;
  data: SSEEventDataMap[T];
}

/**
 * 미확정 SSE 계약 규격 (SSE_CONTRACT)
 */
export const SSE_CONTRACT = {
  VERSION: '0.1.0-draft',
  ENDPOINTS: {
    STREAM: '/chat/stream',
    CONVERSATIONS: '/chat/conversations',
  },
  EVENT_ORDER: ['metadata', 'status', 'delta', 'tool_result', 'done'] as const,
};

/**
 * 개발 및 테스트용 표준 Mock SSE 스트림 Fixture
 */
export const MOCK_SSE_STREAM_FIXTURES: Array<SSEEventFrame<SSEEventType>> = [
  {
    event: 'metadata',
    data: {
      conversation_id: 'conv_20260618_test',
      created_at: '2026-06-18T08:30:00Z',
    },
  },
  {
    event: 'status',
    data: {
      state: 'thinking',
      message: 'Engram 이슈 현황 조회 및 분석 중...',
    },
  },
  {
    event: 'delta',
    data: {
      role: 'assistant',
      content: '안녕하세요! Engram API 페이로드 최적화 작업을 시작하겠습니다. ',
    },
  },
  {
    event: 'status',
    data: {
      state: 'searching',
      message: 'active_caveats 관련 코어 코드 탐색 중',
    },
  },
  {
    event: 'delta',
    data: {
      tool_calls: [
        {
          id: 'call_caveats_fetch',
          type: 'function',
          function: {
            name: 'fetch_active_caveats',
            arguments: '{"sprint_id":4}',
          },
        },
      ],
    },
  },
  {
    event: 'tool_result',
    data: {
      tool_call_id: 'call_caveats_fetch',
      output: '{"status":"success","count":5}',
    },
  },
  {
    event: 'status',
    data: {
      state: 'responding',
      message: '응답 작성 중',
    },
  },
  {
    event: 'delta',
    data: {
      content: '분석 결과 caveats의 detail 본문을 compact 모드에서 생략하도록 코드가 변경되었습니다.',
    },
  },
  {
    event: 'done',
    data: {
      conversation_id: 'conv_20260618_test',
      total_tokens: 412,
      finish_reason: 'stop',
    },
  },
];

/**
 * Mock Stream을 순차적으로 방출(Simulate)하는 헬퍼 함수
 */
export function simulateMockSSE(
  onFrame: <T extends SSEEventType>(frame: SSEEventFrame<T>) => void,
  onEnd?: () => void,
  delayMs = 200
) {
  let index = 0;
  const timer = setInterval(() => {
    if (index < MOCK_SSE_STREAM_FIXTURES.length) {
      onFrame(MOCK_SSE_STREAM_FIXTURES[index]);
      index++;
    } else {
      clearInterval(timer);
      if (onEnd) onEnd();
    }
  }, delayMs);

  return () => clearInterval(timer); // cancel handler
}
