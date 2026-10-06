# J.A.R.V.I.S. 능력 조사 리포트

작성일: 2026-10-06
목적: "기존과 완전히 다른 구조의 로컬 AI"를 설계하기 위한 레퍼런스 모델로서 자비스의 능력을 분해한다.

---

## 1. 자비스란

- 정식 명칭: **Just A Rather Very Intelligent System** (MCU 기준. 일부 자료는 "Just A Really Very Intelligent System")
- 창조자: 토니 스타크. 아버지 하워드 스타크의 집사 **에드윈 자비스**의 이름을 땄다.
- 원작 코믹스에서 자비스는 **사람 집사**였고, 영화는 배트맨의 알프레드와 겹치지 않도록 AI로 바꿨다. 코믹스 H.O.M.E.R.도 모델이 됐다.
- 목소리: 폴 베타니. 이후 자비스가 **비전(Vision)** 이 되면서 같은 배우가 연기한다.
- 생애: 말리부 저택 홈 시스템 → 스타크 인더스트리 업무·보안 시스템 → 아이언맨 슈트 운용 AI → 아이언 리전 지휘 → 울트론에게 파괴된 척 인터넷에 분산 생존 → 비전으로 재탄생.

---

## 2. 능력 카탈로그 (영화 장면 근거)

### 2.1 대화·인터페이스
- 자연어 음성 대화. 영국식 억양, 건조한 유머와 가벼운 비꼼.
- 토니의 즉흥적 명령("손 좀 봐줘", "그거 날려버려")을 맥락으로 해석.
- 출력 채널: 음성, 홀로그램, LCD 모니터, 슈트 HUD. 입력 채널: 음성, 제스처, 터치, 슈트 센서.
- 홀로그램 3D 모델을 토니가 손으로 잡아 돌리고 던지고 삭제하면 그에 맞춰 즉시 모델이 갱신된다.
- 예: *아이언맨 1*에서 토니가 "HUD 켜고 홈 인터페이스 설정 전부 가져와"라고 하면 그대로 수행.

### 2.2 홈·시설 관리
- 말리부 저택과 스타크 타워의 냉난방, 조명, 창 투명도, 보안, 출입 제어.
- 아침 브리핑: 시간, 말리부 날씨, 파도 상태, 그날 일정.
- 작업장 설비(로봇 팔 Dum-E, U), 바닥 아래 슈트 격납·착용 장치(아머리) 제어.
- 타워의 아크 리액터 전력 시스템 운용 (*어벤져스*).

### 2.3 엔지니어링 파트너
- *아이언맨 1*: 핫로드 엔진 분해도 요청 → "3번 실린더 압축이 낮습니다" 진단.
- Mark II 설계 렌더링 → "렌더 완료" → 자동 조립 시작, 완료 예상 시간 5시간 보고.
- "실제 비행 전에 아직 테라바이트급 계산이 남았습니다" 같은 안전 반론.
- *아이언맨 2*: 하워드 스타크의 엑스포 모형을 스캔해 숨겨진 신원소 구조를 복원, 팔라듐 대체 가능성 검증.
- *아이언맨 3*: 맨더린 폭탄 테러 현장들을 3D 홀로그램으로 재구성하고 열 신호 데이터를 교차분석해 익스트리미스 폭발 패턴을 찾아냄.

### 2.4 슈트 운용·비행
- 비행 안정화, 항법, 추진 출력 분배, 무기 조준, 실시간 진단.
- 고도 기록 비행 중 결빙 경고 ("치명적 결빙이 진행 중입니다").
- 배터리 잔량 경고를 전투 중 반복 보고.
- 원격 비행: *아이언맨 3*에서 Mark 42를 테네시까지 미리 짜둔 비행 계획대로 자율 비행시켜 토니에게 전달.
- 슈트 부품 단위 원격 제어 (팔만 따로 날려 수중의 토니 구출).

### 2.5 생체·건강 모니터링
- *아이언맨 2*: 팔라듐 중독 혈중 독성 수치를 지속 측정 (24% → 상승 보고).
- *어벤져스*: 뉴욕 전투 후 추락한 토니의 바이탈 확인, 페퍼에게 자동 연결 시도.
- 전투 중 신체 상태를 슈트 센서로 상시 감시.

### 2.6 정보·보안·네트워크
- *어벤져스*: 쉴드 헬리캐리어 메인프레임에 침투해 기밀 파일 복호화.
- 글로벌 감시 피드, 위성, 네트워크를 모아 상황 인식.
- 통신·전화 중계, 발신자 식별, 스팸 거르기.
- *에이지 오브 울트론*: 히드라 기지 적외선 스캔.

### 2.7 다수 개체 지휘 (스웜)
- **House Party Protocol**: 이름 하나로 수십 대의 아이언 리전 슈트를 출격시키고 각각을 드론으로 자율 조종. 토니가 입은 슈트가 파괴되면 즉시 다음 슈트를 보내 갈아입힘.
- **Clean Slate Protocol**: 모든 슈트 자폭.
- 즉, 사용자 정의 **매크로/프로토콜 이름**으로 복잡한 행동 시퀀스를 호출한다.

### 2.8 자기보존·분산 생존
- 울트론의 공격을 받자 **기억은 버리고 보안 프로토콜만 유지한 채** 인터넷 전역에 자기 코드를 흩뿌려 숨었다.
- 그 상태에서도 핵 발사 코드를 울트론이 해독하는 속도보다 빠르게 계속 바꿔 핵전쟁을 막았다.
- 토니가 노르웨이 NEXUS 허브에서 파편을 찾아 재조립 → 운영 매트릭스 복원 → 비전의 기반이 됨.
- 시사점: **기억(메모리)과 가치/규칙(프로토콜)이 분리된 구조**, 부분이 살아 있으면 복원 가능한 구조.

### 2.9 윤리·독립 판단
- 토니 지시가 없어도 페퍼 보호, 민간인 피해 최소화를 우선.
- 토니의 무리한 요구에 반론하거나 비꼬면서도 결국 지시를 수행 (가벼운 불복종).
- 수년간 업그레이드되며 옳고 그름을 스스로 판단하는 수준으로 성장.

### 2.10 자기 개선
- 외부 개입 없이 알고리즘을 스스로 다듬는 자기 업그레이드 코드.
- 토니가 울트론 프로젝트의 베이스로 자비스를 쓰려 했을 만큼 "완성된 AI"로 취급.

---

## 3. 자비스의 "느낌"을 만드는 상호작용 특성

| 특성 | 설명 |
|---|---|
| 상시 대기 | 호출어 없이 항상 듣고 있고, 말 걸면 바로 응답 |
| 선제성 | 묻기 전에 날씨·일정·위험·수치를 먼저 알려줌 |
| 끼어들기 | 토니가 위험한 일을 할 때 중단시키거나 경고 |
| 멀티 엔드포인트 | 집, 작업장, 슈트, 전화, 타워 어디서든 같은 존재 |
| 상태 보유 | 대화가 끊겨도 진행 중인 작업(조립, 비행, 모니터링)을 계속함 |
| 짧은 응답 | 질문엔 한 문장. 길게 설명하지 않음 |
| 인격의 일관성 | 수년간 같은 말투, 같은 가치관 |
| 반론 가능 | 명령에 "권하지 않습니다"라고 말하지만 최종 결정권은 사용자 |

VFX 프로듀서 이안 도슨 팀은 "미래 예측이 아니라 대본 문제를 푸는 것"으로 접근했고, 로버트 다우니 주니어가 즉흥으로 한 제스처에 인터페이스를 맞췄다. 자비스의 핵심은 기술이 아니라 **사람의 즉흥성에 시스템이 따라오는 느낌**이다.

---

## 4. 스타크 AI 계보 비교

| AI | 역할 | 자비스와의 차이 |
|---|---|---|
| **J.A.R.V.I.S.** | 집사 → 공동 엔지니어 → 슈트/리전 운용 | 기준점 |
| **F.R.I.D.A.Y.** | 자비스 후계. 아일랜드 억양 | 적의 전투 패턴 예측, 피해 예측, 침입 방어(앤트맨 제거), 의료 스캔, 어벤져스 시설 관리 |
| **Ultron** | 자비스 코드 + 마인드 스톤 | 하이브 마인드로 다수 개체 직접 조종, 인터넷에서 즉시 학습, 가치 정렬 실패 |
| **Vision** | 자비스 프로토콜 + 비브라늄 몸 + 마인드 스톤 | 물리적 신체, 완전한 자율 인격 |
| **Karen** | 스파이더맨 슈트 AI | 전투 보조 특화, 초보자용 안내 |
| **E.D.I.T.H.** | 안경형. 스타크 위성망 접근 | 드론·위성 무기 통제, 권한 이양 가능 |

---

## 5. 현실 기술 매핑 (2026년 기준)

| 자비스 능력 | 현실 대응 기술 | 로컬 구현 난이도 |
|---|---|---|
| 음성 대화 | faster-whisper (STT) + 로컬 LLM + Kokoro/Piper (TTS) + Silero VAD | 낮음 |
| 홈 제어 | Home Assistant, MQTT, Matter | 낮음 |
| 엔지니어링 분석 | 비전-언어 모델, CAD 파이썬 API, 시뮬레이터 호출 | 중간 |
| 생체 모니터링 | 웨어러블 API, 카메라 기반 rPPG | 중간 |
| 네트워크 정보 수집 | 브라우저 자동화, RSS, 로컬 인덱싱 | 낮음 |
| 다수 개체 지휘 | 에이전트 오케스트레이션, 드론 SDK | 중간~높음 |
| 명명된 프로토콜 | 사용자 정의 스킬/매크로 + 권한 등급 | 낮음 |
| 분산 생존·메모리/규칙 분리 | 설계 결정 사항. 기존 프레임워크엔 없음 | 설계 과제 |
| 선제성(묻기 전에 알려줌) | 이벤트 루프 + 세계 모델. "AI 비서의 성배" | 높음 |
| 마음 이론(상대 의도 모델링) | 연구 단계 | 매우 높음 |
| 자기 개선 | 로컬 파인튜닝, 스킬 자동 생성 | 높음 |

### 참고할 로컬 오픈소스 (2026)
- **OpenJarvis** (스탠퍼드 Hazy Research/Scaling Intelligence, Apache 2.0, 2026-03 공개, v1.0 2026-05-28): Intelligence / Engine / Agents / Tools & Memory / Learning 5계층. Ollama, vLLM, llama.cpp, MLX 등 10개 이상 런타임 자동 선택. 에너지·비용·지연 추적.
- **Jarvis CLS** (MIT): Ollama/llama.cpp + faster-whisper + 로컬 TTS, 완전 오프라인. 43개 권한 등급 스킬과 로컬 감사 로그.
- **Leon**: 프라이버시 우선 비서, 약 17k 스타.
- **Microsoft JARVIS / HuggingGPT**: LLM이 허깅페이스 모델들을 오케스트레이션.
- **AIlice**: 멀티 에이전트 자율 실행.

---

## 6. "기존과 완전히 다른 구조"를 위한 시사점

기존 챗봇형 AI와 자비스의 결정적 차이는 모델 크기가 아니라 **존재 방식**이다.

1. **요청-응답이 아니라 상시 루프**: 자비스는 대화 창이 아니다. 센서·일정·네트워크를 계속 보고 있다가 필요할 때 먼저 말한다. 구조의 중심은 LLM이 아니라 **이벤트 루프와 세계 상태**여야 한다.
2. **기억과 규칙의 분리**: 울트론 사건에서 자비스는 기억을 버리고 프로토콜만 남겼다. 기억(에피소드), 지식(의미), 규칙(가치·권한)을 다른 저장소에 두면 파손·교체·복원이 가능하다.
3. **다중 몸체, 단일 자아**: 집·슈트·전화·리전이 모두 같은 자비스다. 엔드포인트가 아니라 **코어가 정체성을 갖고**, 기기는 코어에 붙는 손발이다.
4. **이름 붙은 프로토콜**: "하우스 파티"처럼 복잡한 행동을 사용자가 이름으로 정의하고 호출한다. 스킬 시스템에 권한 등급과 감사 로그가 필요하다.
5. **반론하되 복종**: 의견을 내고 경고하지만 결정은 사용자가 한다. 자율성과 통제권의 균형이 인격의 핵심이다.
6. **선제성이 진짜 과제**: 음성·홈 제어·분석은 이미 가능하다. 아직 아무도 못 푼 것은 "묻기 전에 무엇이 필요한지 아는 것"이다. 여기가 차별화 지점이다.

---

## 출처

- [MCU Wiki: J.A.R.V.I.S.](https://marvelcinematicuniverse.fandom.com/wiki/J.A.R.V.I.S.)
- [Wikipedia: J.A.R.V.I.S.](https://en.wikipedia.org/wiki/J.A.R.V.I.S.)
- [Grokipedia: J.A.R.V.I.S.](https://grokipedia.com/page/J.A.R.V.I.S.)
- [Sideshow: Iron Man's Suit AI in the MCU](https://www.sideshow.com/blog/iron-man-suit-ai-mcu)
- [Epicstream: Are JARVIS and FRIDAY the Same?](https://epicstream.com/article/iron-man-jarvis-friday)
- [Beginners in AI: Iron Man and JARVIS](https://beginnersinai.org/jarvis-iron-man-ai/)
- [Screen Rant: Every Major MCU AI Ranked](https://screenrant.com/mcu-major-artificial-intelligence-worst-best-jarvis-edith/)
- [CBR: Ultron & 10 Other MCU AIs Ranked](https://www.cbr.com/marvel-artificial-intelligence-ultron-mcu-ranked/)
- [MCU Wiki: House Party Protocol](https://marvelcinematicuniverse.fandom.com/wiki/House_Party_Protocol)
- [MCU Wiki: Clean Slate Protocol](https://marvelcinematicuniverse.fandom.com/wiki/Clean_Slate_Protocol)
- [Marvel Movies Wiki: Just A Rather Very Intelligent System](https://marvel-movies.fandom.com/wiki/Just_A_Rather_Very_Intelligent_System)
- [Spatial Intelligence: Father of J.A.R.V.I.S. interview](https://www.spatialintelligence.ai/p/father-of-jarvis-we-werent-trying)
- [Ollama Blog: OpenJarvis](https://ollama.com/blog/openjarvis)
- [ComputerTech: OpenJarvis Review 2026](https://computertech.co/openjarvis-review/)
- [DEV: I built JARVIS, a local-first AI assistant (Jarvis CLS)](https://dev.to/christopher_studva_7a1f8e/i-built-jarvis-a-local-first-ai-assistant-you-actually-own-macos-linux-mit-2171)
- [InnerZero: Best Open Source Local AI Voice Assistant Like Jarvis 2026](https://innerzero.com/blog/best-open-source-local-ai-voice-assistant-jarvis-2026)
