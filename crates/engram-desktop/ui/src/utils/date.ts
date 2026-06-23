/**
 * SQLite 등에서 타임존 표시(Z) 없이 내려오는 UTC 날짜 문자열을
 * 브라우저에서 올바르게 UTC 기준으로 해석하도록 접미사 'Z'를 붙여 Date 객체로 파싱합니다.
 */
export function parseUTCDate(dateStr: string | null | undefined): Date {
  if (!dateStr) return new Date();
  
  let s = dateStr.trim();
  // 공백이 있을 경우 ISO 표준인 'T'로 치환합니다. (예: "2026-06-23 05:45:08" -> "2026-06-23T05:45:08")
  s = s.replace(' ', 'T');
  
  // 타임존 표시가 없으면 Z(UTC)를 붙여줍니다.
  if (!s.endsWith('Z') && !s.includes('+') && !/-\d{2}:\d{2}$/.test(s)) {
    s += 'Z';
  }
  
  return new Date(s);
}
