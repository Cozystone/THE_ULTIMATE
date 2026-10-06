# 비(非)스케일링 AI 아키텍처 현황 조사 (2026년 10월 기준): 발달 로보틱스, 내재적 동기, 월드모델, 능동추론, 신경기호, 지속/테스트타임 학습, 생물학적 대안, 소형 로컬 에이전트

> 작성 기준일: 2026-10-06. 모든 주장에 날짜를 표기했고, "논문에서 시연(demonstrated in paper)" / "독립 재현(independently reproduced)" / "주장(claimed)" 을 구분했다. 목표: Ryzen 9 + RTX 5080 16GB(상한) ~ CPU 전용 노트북(하한)에서 돌아가며, 사용자 환경을 이해하고 선제적으로 행동하는 지속학습 에이전트("Jarvis")를 만들기 위해 **단 하나의 경로**를 고르는 데 필요한 근거를 정리한다.
>
> 조사 방법 메모: 1차 출처(arXiv 원문/저장소/공식 블로그)를 우선 인용했고, 2차 출처(보도·블로그)는 그렇게 표시했다. 세션의 웹검색 한도(200회)를 모두 소진한 뒤에는 알려진 URL을 직접 가져오는 방식으로 보완했다. 확인 못 한 항목은 각 절 Gaps에 남겼다.

---

## 1. 발달 로보틱스와 내재적 동기(Oudeyer 학습진전 호기심, 목표 배블링, IMGEP, 오토텔릭 에이전트, Schmidhuber 인공 호기심, iCub/Baby Sophia/BabyBench): 장난감 환경을 넘어 무엇이 확장되었는가

### Takeaway
학습진전(learning progress, LP) 기반 호기심과 오토텔릭(autotelic) 목표 생성은 2025년에 **LLM 에이전트의 목표 선택기(MAGELLAN, ICML 2025)**와 **ARC 프로그램 합성의 자기개선 루프(SOAR, ICML 2025)** 로 "장난감을 넘어서" 확장된 유일한 축이며, 순수 신체 발달 벤치마크(BabyBench 2025)는 여전히 시뮬레이션 유아 모델(MIMo)에서 자기접촉·손 응시 같은 초기 행동 단계에 머물러 있다. 실제 로봇(iCub) 2025-26년 성과는 신뢰할 만한 출처를 찾지 못했다.

### Cited Findings
- 오토텔릭 에이전트 정의: "스스로의 문제(목표)를 표현·생성·선택·해결하는 법을 배우는 내재적 동기 에이전트" — Colas, Karch, Sigaud, Oudeyer, JAIR vol.74 서베이 — [Source](https://dl.acm.org/doi/10.1613/jair.1.13554)
- IMGEP(Intrinsically Motivated Goal Exploration Processes)의 자동 커리큘럼 학습 정식화(JMLR 2022, vol.23) — [Source](https://www.jmlr.org/papers/volume23/21-0808/21-0808.pdf)
- MAGELLAN (ICML 2025, Gaven·Carta·Romac·Colas·Lamprier·Sigaud·Oudeyer): LLM 에이전트가 "자신의 역량과 학습진전을 온라인으로 예측"하는 메타인지 프레임워크. 목표 간 의미 관계를 포착해 "표본 효율적인 LP 추정"과 "진화하는 목표 공간에의 동적 적응"을 가능케 했고, 저자들은 "크고 진화하는 목표 공간을 완전히 마스터한 유일한 방법"이라고 주장(논문 내 시연). 코드 공개(flowersteam/MAGELLAN) — [Source](https://arxiv.org/abs/2502.07709); [Source](https://icml.cc/virtual/2025/poster/44419); [Source](https://github.com/flowersteam/MAGELLAN)
- SOAR (ICML 2025, Pourcel·Colas·Oudeyer): LLM을 "진화적 탐색 + 사후(hindsight) 학습" 루프에 넣어 프로그램 합성을 자기개선. ARC-AGI-1 공개 테스트셋 52% 달성(논문 내 시연). 실험 모델은 Qwen-2.5-Coder-14B 계열이며, 미세조정 LLM·생성 프로그램 500만 개 데이터셋·코드를 공개 — [Source](https://arxiv.org/abs/2507.14172); [Source](https://huggingface.co/julien31/Soar-qwen-7b). ARC Prize 2025 논문상 2위($20k) — [Source](https://arcprize.org/blog/arc-prize-2025-results-analysis)
- Schmidhuber: 1990-91년 이후 "인공 호기심·창의성" 보고서의 2025년 업데이트판 공개(35주년); NeurIPS 2025에 "Curious Causality-Seeking Agents Learn Meta Causal World"(Zhao 외, Schmidhuber 공저) 발표 — [Source](https://people.idsia.ch/~juergen/artificial-curiosity-since-1990.html); [Source](https://people.idsia.ch/~juergen/blog.html)
- BabyBench 2025 (IEEE ICDL 2025, 프라하): 다중감각 유아 모델 MIMo에 "외적 보상 없이, 외부 감독 없이" 자기접촉(self-touch)과 손 응시(hand regard)를 학습시키는 대회. 제출 마감 2025-09-08, 발표 2025-09-19, 상금 €150. 우승 Lucas Yang(UCL)·Daniela Corbetta(U. Tennessee)·Lorenzo Jamone(UCL); 결선 Yordanova(HU Berlin), Wu(UCSD). 우승 방법·점수는 홈페이지에 미기재 — [Source](https://babybench.github.io/2025/); [Source](https://icdl2025.fel.cvut.cz/babybench/); [Source](https://github.com/babybench/BabyBench2025_Starter_Kit)
- Baby Sophia (arXiv 2511.09727, 2025-11): BabyBench 환경에서 "유아의 호기심 기반 신체 탐색을 모사하는 내재적 보상"으로 RL. 커리큘럼으로 자기접촉, 모터 배블링+시각-운동 협응으로 손 응시 학습. 5쪽 논문, 계산량·한계 미기재(논문 내 시연) — [Source](https://arxiv.org/abs/2511.09727)
- 2026-06 "The Tao of Agency: Autotelic AI, Embedded Agency and Dissolution of the Self"(Sarkar): 오토텔릭 에이전시에서 "임베디드성(embeddedness)은 필요조건이지만 충분조건이 아니다"라는 개념 논문 — [Source](https://arxiv.org/abs/2606.19924)
- DARPA L2M 생물 영감 결과(2019): USC 팀이 "5분간의 비구조적 놀이(unstructured play)" 만으로 걷기를 학습하는 로봇 다리 시연(프로그램 내 시연) — [Source](https://www.darpa.mil/news/2019/lifelong-learning-machines)
- MIT Cog 프로젝트 회고(Brooks 외): "기존 하위 시스템·하위 행동들로부터 일관된 전역 행동을 시연하는 것이 빠진 핵심 조각"이었고, 액추에이터 경쟁과 "세계를 통한 의도치 않은 결합"이 간섭을 일으켰다고 기술 — [Source](https://people.csail.mit.edu/brooks/papers/CMAA-group.pdf)

### Inferences
- "장난감을 넘어선" 확장은 **몸이 아니라 LLM/프로그램 공간**에서 일어났다: LP 기반 목표 선택(MAGELLAN)과 자기개선 탐색(SOAR)은 로컬 소형 LLM 위에 얹을 수 있는 **모듈**이며, 14B 이하 모델로 실험되었다. 신체 발달 축(BabyBench/Baby Sophia)은 2025년에도 "자기접촉" 수준이므로 Jarvis 용도로는 직접 쓸 결과가 없다.
- 학습진전(LP) 지표는 "사용자 환경에서 무엇을 더 배울지"를 고르는 선제적 행동의 근거로 재활용 가능하다(사용자 작업 패턴에 대한 예측 오차의 감소율을 LP로 보는 식). 다만 이것은 조사로 확인된 것이 아니라 추론이다.

### Gaps
- iCub 2025-2026년 발달학습 결과는 검색에서 신뢰할 만한 1차 출처를 찾지 못했다(검색 결과가 2010년대 자료에 머묾).
- BabyBench 2025 우승 방법/점수, 평가 지표의 정량 정의는 공개 페이지에 없었다(스타터 킷 저장소를 직접 열어야 함).
- MAGELLAN의 환경 이름·목표 공간 크기·사용 LLM 크기는 초록에서 확인 불가(본문 열람 필요).
- "Colas 외 오토텔릭 에이전트"의 2026년 신규 결과는 찾지 못했다.

---

## 2. 월드모델(V-JEPA 2/2.1, LeJEPA, LeWorldModel, AMI Labs, DreamerV3/Dreamer 4, Genie 3, NVIDIA Cosmos): 공개 가중치·VRAM·개인 데이터로 처음부터 학습 가능성

### Takeaway
2026년 10월 기준 **소비자 GPU 1장(심지어 CPU)으로 몇 시간 안에 처음부터 학습 가능한 잠재 예측 월드모델(LeWorldModel, ~15M 파라미터)** 이 코드·체크포인트와 함께 존재하지만, 독립 재현 연구(2026-08)는 결과가 **문서화되지 않은 평가 규약에 극도로 민감**함을 보였다. 대형 월드모델(V-JEPA 2-AC 1B, Dreamer 4 2B, Genie 3, Cosmos)은 실제 로봇 조작 또는 실시간 시뮬레이션을 시연했으나, 학습은 수백 TPU/H100 규모이고 추론도 RTX 4090에서 행동당 16초(V-JEPA 2-AC) 수준이다.

### Cited Findings
**V-JEPA 2 / 2.1 (Meta, 2025-06 공개)**
- 100만 시간 이상 인터넷 비디오로 사전학습. SSv2 77.3% top-1, Epic-Kitchens-100 행동 예측 39.7 R@5, 8B LLM 정렬 후 PerceptionTest 84.0 / TempCompass 76.9. V-JEPA 2-AC는 "62시간 미만의 라벨 없는 로봇 비디오(DROID)"로 후속학습 후 **두 연구실의 Franka 팔에 제로샷 배치**(논문 내 시연) — [Source](https://arxiv.org/abs/2506.09985)
- 로봇 결과(두 연구실 평균): reach 100%, grasp 컵 65%/상자 25%, pick-and-place 컵 80%/상자 65%. 계획은 CEM으로 **RTX 4090 1장에서 행동당 약 16초**. 베이스라인 Octo는 grasp 15%/pick-place 10%; Cosmos는 "행동 하나에 4분". 명시된 한계: 카메라 위치 민감성(수동 조정 필요), 자기회귀 예측의 오차 누적으로 장기 과제 한계, 언어가 아닌 **시각적 목표**만 지원 — [Source](https://arxiv.org/html/2506.09985)
- 공개 체크포인트: ViT-L/16 300M, ViT-H/16 600M, ViT-g/16 1B(256/384px); V-JEPA 2.1: ViT-B 80M, ViT-L 300M, ViT-g 1B, ViT-G 2B(384px); V-JEPA 2-AC ViT-g 체크포인트. 라이선스 대부분 MIT(일부 Apache 2.0). 추론 VRAM 요구는 문서에 명시 없음 — [Source](https://github.com/facebookresearch/vjepa2)
- PSG-JEPA (arXiv 2608.06799, 2026-08): "순방향 예측 목적함수는 로봇 중심 물리 상태의 신뢰할 수 있는 식별가능성을 명시적으로 강제하지 않는다"며, 고유수용감각·관절각 변화에 잠재를 접지(grounding)하는 보조 목적을 추가해 계획 성능을 높였다고 보고(논문 내 시연) — [Source](https://arxiv.org/abs/2608.06799)

**LeJEPA → LeWorldModel → AMI Labs**
- LeJEPA (arXiv 2511.08544, 2025-11-11, Balestriero·LeCun): 임베딩의 최적 분포가 등방 가우시안임을 증명하고 SIGReg(선형 복잡도 분포 정합) 도입; 60개 이상 모델에서 안정 학습, ViT-H/14 ImageNet-1k 선형평가 약 79% — [Source](https://arxiv.org/abs/2511.08544); [Source(2차)](https://the-decoder.com/yann-lecun-unveils-lejepa-likely-his-final-meta-project-before-launching-a-startup/)
- LeWorldModel (2026-03-13 공개, Mila/NYU/Samsung SAIL 공동): "~15M 파라미터, **단일 GPU에서 몇 시간** 학습", "파운데이션 모델 기반 월드모델보다 48배 빠른 계획"(DINO-WM 약 47초 vs 약 1초). 환경 4종: Two-Room, Reacher, Push-T, OGBench-Cube. PLDM을 능가, Push-T·Reacher에서는 사전학습 특징 없이 DINO-WM 능가, 시각적으로 복잡한 OGBench-Cube에서는 DINO-WM이 우세. 논문·코드·데이터·체크포인트 공개. 한계: Two-Room에서 저조("과제의 내재 차원이 너무 낮아 가우시안 정규화를 방해") — [Source](https://le-wm.github.io/)
- **독립 재현(arXiv 2608.10145, 2026-08, J. Singh)**: 원논문 TwoRoom 약 87% → 저장소 설정으로 94.0% 재현했으나 **저자 체크포인트는 재현 규약에서 84.0%**; 결과를 좌우하는 4가지 규약(밀집 행동 수집, 인코더 폭 설정, ImageNet 정규화, 행동 z-score)이 "공개된 어떤 설정 파일에도 없음"; 부록과 저장소의 목표 오프셋·스텝 예산이 달라 같은 가중치가 14.0% vs 84.0%; 목표 구성만 바꾸면 84.0%→8.0%; "**1스텝 예측 정확도는 장기 계획 성공을 예측하지 못함**"(7배 오차 범위의 체크포인트들에서); BatchNorm이 검증 손실을 최대 300배 부풀려 "학습 손실이 내내 평평했음"을 감춤 — [Source](https://arxiv.org/abs/2608.10145)
- jepa-studio (커뮤니티 도구, GitHub 스타 0): 자기 이미지/비디오/시계열로 LeJEPA·LeWM을 로컬 학습, CPU/CUDA/ROCm/MPS 및 브라우저 WebGPU 지원, "2코어 CPU, GPU 없음" 예시 결과 포함; CEM 계획 봇이 문 통과 80%, 블록 밀기 25% 성공(개발자 자체 보고, 미검증) — [Source](https://github.com/Normansrule/jepa-studio)
- AMI Labs: 2026-03-09 보도로 **$1.03B 시드, $3.5B 프리머니**; COO Laurent Solly, CSO Saining Xie, Pascale Fung, VP 월드모델 Michael Rabbat; 파리 본사 + 뉴욕·몬트리올·싱가포르 — [Source(2차)](https://techcrunch.com/2026/03/09/yann-lecuns-ami-labs-raises-1-03-billion-to-build-world-models/)
- "When Does LeJEPA Learn a World Model?"(Klindt·LeCun·Balestriero, 2026-05-25 프리프린트, 2차 보도): 잠재변수가 **가우시안이고 정상(stationary) 가산잡음 전이**를 따를 때 LeJEPA가 잠재변수를 선형변환까지 복원(선형 식별가능성)함을 증명; 가우시안 조건은 "유일하게 필요"하며, 보도는 "현실의 비정상·복잡성에서는 성립하지 않을 수 있다"고 지적 — [Source(2차)](https://www.startuphub.ai/ai-news/ai-figures/2026/figure-yann-lecun-jepa-formal-proof-world-models-2026-08-04)

**Dreamer 계열**
- DreamerV3: "150개 이상의 다양한 과제에서 단일 설정으로 전문 방법을 능가", "사람 데이터·커리큘럼 없이 처음부터 마인크래프트 다이아몬드를 수집한 최초 알고리즘"; Nature 2025 게재("Mastering diverse control tasks through world models", Hafner 외); 코드 MIT, 단일 GPU 학습, CPU 플랫폼 플래그 지원 — [Source](https://arxiv.org/abs/2301.04104); [Source](https://github.com/danijar/dreamerv3)
- Dreamer 4 (arXiv 2509.24527, 2025-09): 토크나이저 400M + 다이내믹스 1.6B(총 2B); **256–1024 TPU-v5p** 로 학습; VPT 계약자 데이터셋 2,500시간; 오프라인 데이터만으로 **다이아몬드 획득 에피소드 0.7%**, 중간 이정표(돌 곡괭이 등) 90% 이상; shortcut forcing(K=4 샘플 스텝)으로 "단일 GPU 실시간 상호작용 추론" 주장(FPS 미기재); 코드·가중치 공개 언급 없음, github.com/danijar/dreamer4 는 2026-10-06 현재 404 — [Source](https://arxiv.org/html/2509.24527); [Source](https://danijar.com/project/dreamer4/)

**Genie 3 / Cosmos / 기타 소형 월드모델**
- Genie 3 (DeepMind, 2025-08): 텍스트 → 실시간 24fps·720p 탐색 가능 환경, "몇 분간" 일관성 유지; 프레임을 자기회귀 생성 — [Source](https://deepmind.google/blog/genie-3-a-new-frontier-for-world-models/). 2026-01-29 "Project Genie"를 AI Ultra 구독자에게 공개; 공개 API·가중치 없음(2차: 월 $250, 미국 한정) — [Source](https://en.wikipedia.org/wiki/Genie_(world_model)); [Source(2차)](https://meetcody.ai/models/genie-3/)
- NVIDIA Cosmos(공개 가중치): Predict 7B(Text2World/Video2World) H100 단일 GPU 추론에 **42 GiB**, 14B **68 GiB**; 오프로딩 시 약 39GB; 14B Video2World 56.38GB로 RTX 4090/5090(24/32GB) 초과; Cosmos 3 Nano(16B)는 96GB급 필요 — [Source(2차)](https://www.spheron.network/blog/deploy-nvidia-cosmos-gpu-cloud-synthetic-data/); [Source](https://github.com/nvidia/Cosmos); [Source](https://arxiv.org/html/2501.03575v1)
- SANA-WM (NVIDIA, 2026-05 보도): 2.6B 오픈소스, 이미지 1장+카메라 궤적 → 60초 720p; **학습은 H100 64장**, 추론은 단일 GPU — [Source(2차)](https://www.marktechpost.com/2026/05/16/nvidia-introduces-sana-wm-a-2-6b-parameter-open-source-world-model-that-generates-minute-scale-720p-video-on-a-single-gpu/)
- MiniWorld (arXiv 2608.01127): 블록-인과 디퓨전 트랜스포머로 스트리밍 비디오 월드모델을 처음부터 학습, "단일 8-GPU 서버에서 며칠" — [Source](https://huggingface.co/papers/2608.01127)
- 개인 개발자 사례: RTX 3080 Ti 1장으로 약 12.5M 파라미터 모델을 포켓몬 레드에 학습, "다음에 무엇이 일어날지 예측"하며 버튼 의미를 발견(2차 보도, 기사 본문 접근 실패로 세부 미확인) — [Source(2차)](https://www.tomshardware.com/tech-industry/artificial-intelligence/developer-trains-a-small-ai-on-a-single-rtx-3080-ti-gaming-gpu-to-play-pokemon-red-model-discovered-what-each-button-does-by-predicting-what-happens-next)

### Inferences
- RTX 5080 16GB에서 현실적인 것: (1) V-JEPA 2 ViT-L(300M)/ViT-g(1B) **고정 인코더 추론**(fp16 기준 수 GB), (2) LeWM급 15M 잠재 예측기를 **사용자 데이터로 처음부터 학습**(수 시간), (3) DreamerV3 소형 설정. 불가능한 것: Cosmos 7B+ 생성(42GB+), Dreamer 4 재학습, Genie 3(비공개).
- "개인 데이터 스트림으로 처음부터 학습"은 LeWM/jepa-studio가 기술적으로 열어 놓았으나, 독립 재현이 보여준 **평가 규약 민감성과 "1스텝 예측 정확도 ≠ 계획 성공"** 문제는 Jarvis용 월드모델의 **유용성 검증 지표를 먼저 정의**해야 함을 뜻한다.
- 월드모델 계열은 "물리/화면 동역학 예측"에는 근거가 있으나, "사용자 의도 이해·언어 목표"는 V-JEPA 2-AC 저자 스스로 한계로 명시했다(시각 목표만). 언어 기관은 별도로 필요하다.

### Gaps
- V-JEPA 2 사전학습 GPU 수/시간, 추론 VRAM 공식 수치 미확인(초록·README에 없음).
- Dreamer 4 코드/가중치 공개 여부를 공식 출처로 확정하지 못함(저장소 404, 프로젝트 페이지 언급 없음).
- "When Does LeJEPA Learn a World Model?"의 arXiv ID와 원문을 확인하지 못했다(2차 보도만).
- AMI Labs의 2026년 공개 논문 목록(LeWM·LeJEPA 외)과 제품 로드맵은 1차 출처로 확인 불가.
- Genie 3 가격·지역 제한은 2차 출처뿐.

---

## 3. 대규모 능동추론(deep active inference, VERSES AXIOM/Genius, pymdp): 주장과 독립 평가

### Takeaway
AXIOM(2025-05)은 **0.3–1.6M 파라미터로 A100 1장에서 10k 스텝 만에** VERSES가 직접 만든 단순 스프라이트 게임 10종(Gameworld 10k)에서 BBF·DreamerV3를 앞섰다고 보고했고 코드를 공개했지만, 저자 스스로 "핵심 사전지식이 자율 발견이 아닌 수작업 설계"이며 "Atari·Minecraft 같은 복잡한 영역으로 확장 불가"라고 인정했다. 학계의 독립 재현은 발견하지 못했고, VERSES가 인용하는 "독립 검토"는 의뢰받은 인증 회사의 평가다.

### Cited Findings
- AXIOM 방법: 장면을 객체 구성으로 표현, "희소한 객체-객체 상호작용을 포착하는 구간별 선형 궤적", 이벤트마다 혼합모델이 온라인 확장, 주기적 **베이지안 모델 축소(BMR)** 로 일반화; "경사 기반 최적화의 계산 비용 없이" — [Source](https://arxiv.org/abs/2505.24784)
- Gameworld 10k는 **VERSES가 이 논문을 위해 만든 벤치마크**: 10개 게임(Aviate, Bounce, Cross, Driver, Explode, Fruits, Gold, Hunt, Impact, Jump)은 "LLM의 도움으로 생성", "단색 스프라이트" 사용. 10k 스텝 누적보상(일부): Hunt 206±20 vs BBF 4±12 vs DreamerV3 6±9; Gold 190 vs −26 vs −21; Fruits 182 vs 86 vs 60; Explode 180 vs 101 vs 35; Aviate −90 vs −90 vs −114. 파라미터 0.3–1.6M vs BBF 6.47M vs DreamerV3 420M; 스텝당 모델 갱신 18ms vs 135 vs 221ms; 계획 252–534ms vs DreamerV3 823ms(A100 1장). 저자 명시 한계: "핵심 사전지식이 자율적으로 발견되지 않고 설계됨", Atari/Minecraft 같은 "생성 과정이 불투명한" 영역으로 현재 확장 불가, "의도적으로 단순화한 시각 요소" — [Source](https://arxiv.org/html/2505.24784v1)
- VERSES 보도자료(2025-06): 정규화 점수 77 vs 48(DreamerV3), 7.6배 표본효율(3,175 vs 24,207 스텝), GPU 시간 39배(약 10분 vs 370분), 비용 12배($0.66 vs $25.54) — 주장(자체 벤치마크) — [Source](https://www.verses.ai/news/verses-digital-brain-beats-googles-top-ai-at-gameworld-10k-atari-challenge)
- 코드: GitHub VersesTech/axiom, **"VERSES Academic Research License"(상업적 사용·파생물 제한)**, Nvidia GPU(CUDA 12) 권장·CPU 지원(성능 저하), Python 3.11 — [Source](https://github.com/VersesTech/axiom)
- "독립 검토": VERSES가 논문·증명·코드를 Soothsayer Analytics(인증·자문 회사)에 제출해 평가받았다고 보도 — 의뢰 평가이지 학술 재현이 아님 — [Source(2차)](https://soothsayeranalytics.com/media/soothsayer-assesses-verses-axiom-model-a-step-forward-in-efficiency-and-learning-in-ai); [Source](https://www.verses.ai/blog/whitepaper-mastering-gameworld-10k-in-minutes-with-the-axiom-digital-brain)
- Genius 상용화: 1,500건 이상 가입 요청(2025); 2026-04 Prodigii AI와 Genius+AXIOM 라이선스 계약(최대 $10M 매출 전망, 2차) — [Source](https://www.verses.ai/news/verses-provides-update-on-genius-adoption); [Source(2차)](https://www.stocktitan.net/news/VRSSD/prodigii-licenses-verses-c6dkhipk498d.html)
- pymdp: 이산 상태공간 POMDP 능동추론 라이브러리(JOSS) — 소규모 이산 문제용 — [Source](https://joss.theoj.org/papers/10.21105/joss.04098)
- Deep AIF for delayed/long-horizon (arXiv 2505.19867, 2025): "실용적 AIF 에이전트는 여전히 정확한 즉시 예측과 전수 계획에 의존하며 지연 환경에서 악화"된다고 인정; 생성 정책으로 "수십~수백 스텝" 전망, "산업 시나리오 모사 환경"에서 평가; 초록에 RL 베이스라인 대비 정량치 없음 — [Source](https://arxiv.org/abs/2505.19867)

### Inferences
- AXIOM의 진짜 기여는 "객체 중심 혼합모델 + BMR"이 **소형 파라미터로 빠른 온라인 학습**을 한다는 점이며, 이는 CPU 전용 노트북에서도 돌아갈 가능성이 높다(저장소가 CPU 모드를 제공). 그러나 입력이 단색 스프라이트 객체로 사전 분할되어야 하므로 **실제 화면·카메라 픽셀을 바로 넣을 수 없다**.
- 라이선스가 학술용이라 개인 Jarvis에 포함하려면 재구현이 필요하다.

### Gaps
- AXIOM에 대한 학술적 독립 재현/비판 논문을 찾지 못했다(검색상 부재).
- Genius 플랫폼의 기술적 실체(AXIOM이 실제 제품에 어떻게 쓰이는지), 2026년 벤치마크는 1차 출처 부재.
- pymdp/deep AIF가 Atari 100k 등 표준 벤치마크에서 DreamerV3급 성능을 낸 기록은 찾지 못했다.

---

## 4. 신경기호·프로그램 합성(DreamCoder 계열, ARC Prize 2025-2026, HRM/TRM/CompressARC, Ndea, Lake & Baroni MLC): 일상적 접지 추론에 무엇이 작동하는가

### Takeaway
ARC-AGI-2(2025)는 **소형 모델 + 테스트타임 훈련(TTT) + 합성 데이터**(NVARC: Qwen-4B 기반, 과제당 ~$0.20, 24.03%)와 **7M~76K 파라미터 과제별 최적화**(TRM 45%/CompressARC 20% on ARC-AGI-1, RTX 4070에서 퍼즐당 20분)가 소비자 하드웨어에서 가능함을 보였지만, ARC Prize와 후속 분석은 이들이 **"순수 전이적(transductive)"이고 과제 ID·증강·투표에 의존**함을 밝혔다. 2026년 ARC-AGI-3(상호작용)에서 100%를 찍은 시스템(Tycho, NVIDIA AVO)은 모두 **프론티어 LLM 코딩 에이전트가 실행 가능한 월드모델을 합성**하는 방식이다.

### Cited Findings
- ARC Prize 2025(2025-03-26~11-03): 1,455팀 15,154 제출. 1위 NVARC 24.03%, 2위 ARChitects 16.53%(2D 인식 마스크드 디퓨전 LM + 재귀적 자기정제), 3위 MindsAI 12.64%(TTT 파이프라인), 4위 6.67%, 5위 6.53% — [Source](https://arcprize.org/blog/arc-prize-2025-results-analysis); [Source](https://arxiv.org/html/2601.10904v1)
- Kaggle 제약: "과제당 약 20센트", "소형 GPU 4장에서 최대 12시간". NVARC는 **Qwen-4B** 를 주 모델로 사용, GPT-OSS로 26만 개 퍼즐 설명을 합성(20개 구현 중 8개 이상 동일 출력일 때 채택), 사후 실험에서 TRM pass@128 약 30% — [Source(2차)](https://trelis.substack.com/p/nvarc-2025-arc-prize-winners)
- 검증된 프론티어: Opus 4.5(thinking 64k) 37.6% @ $2.20/과제; Gemini 3 Pro + Poetiq 정제 54% @ $30/과제(기본 31%) — [Source](https://arcprize.org/blog/arc-prize-2025-results-analysis)
- 논문상: 1위 TRM(7M, ARC-AGI-1 45%, ARC-AGI-2 8%, $50k); 2위 SOAR(52% public ARC-AGI-1); 3위 CompressARC(**76K 파라미터, 사전학습 없음**, ARC-AGI-1 20%, **RTX 4070 1장에서 퍼즐당 약 20분**) — [Source](https://arcprize.org/blog/arc-prize-2025-results-analysis); [Source](https://arxiv.org/abs/2512.06104); [Source](https://arxiv.org/pdf/2510.04871)
- 조직위 결론: 2025년의 정의적 패턴은 "정제 루프(refinement loops)"; "현재 프론티어 AI의 추론 성능은 근본적으로 지식 커버리지에 제약"; 인간 테스터 97–98%, ARC-AGI-2 전 과제가 2인 이상 인간에 의해 해결 — [Source](https://arxiv.org/html/2601.10904v1)
- HRM 독립 검증(ARC Prize): ARC-AGI-1 **32%**(주장 41%; 9h16m, $148.50), ARC-AGI-2 2%(12h35m, $201). "하이퍼파라미터 최적화 없이도 일반 트랜스포머가 HRM의 ~5pp 이내"; 성능은 외부 정제 루프가 주도(1회 정제로 +13pp); 평가 과제만으로 학습 시 31% vs 41% → "성능의 대부분이 평가 시점에 본 과제 학습에서 기인"; 300회 증강으로 거의 최대; "순수 전이적", 퍼즐 임베딩 의존 — [Source](https://arcprize.org/blog/hrm-analysis)
- TRM 비판(arXiv 2512.11847): 1000샘플 투표가 Pass@1을 약 11pp 끌어올림; **퍼즐 ID를 공백/무작위 토큰으로 바꾸면 정확도 0**; 대부분의 정확도가 첫 재귀 스텝에서 달성 → "효율·과제 조건화·공격적 테스트타임 계산의 상호작용이지 깊은 내부 추론이 아님" — [Source](https://arxiv.org/abs/2512.11847)
- ARC-AGI-3(2026-03-25 공개): 지시 없는 턴제 상호작용 환경, "인간은 100% 해결, 2026-03 기준 프론티어 AI는 1% 미만"; 인간 행동 기준선 대비 효율 점수; 탐색·목표 추론·내부 모델 구축·계획을 측정하며 언어·외부 지식 배제 — [Source](https://arxiv.org/abs/2603.24621). 2026 대회 ARC-AGI-3 트랙 상금 $850k, 완전 비공개셋 100% 시 $700k(2차) — [Source(2차)](https://www.datacamp.com/blog/arc-agi-3)
- Tycho(arXiv 2607.28287, 2026-07): 코딩 에이전트가 게임을 "렌더링된 결정적 Moore 기계"로 모델링; Claude Opus 4.8로 25개 공개 게임 평균 RHAE 88.49(베이스라인 79.07); GPT-5.6 Sol·Opus 5로 **100.00 RHAE, 183레벨 전부 완료** — [Source](https://arxiv.org/abs/2607.28287)
- NVIDIA AVO(2026-08-21): "범용 코딩 에이전트 시스템"(LLM 스캐폴드), Claude Opus 5로 공개셋 25환경 100.00 RHAE; "준비공개·완전비공개 대회셋 결과가 아님"; 비용 미공개; "통제된 ablation이 아님" — [Source](https://developer.nvidia.com/blog/nvidia-avo-reaches-100-on-arc-agi-3-demonstrating-a-frontier-level-general-purpose-architecture-for-long-horizon-autonomous-agents/)
- 프리뷰 단계 비-LLM 결과: Tufa Labs의 StochasticGoose(CNN+RL) 12.58%(2차 블로그, 미검증) — [Source(2차)](https://medium.com/@AdithyaGiridharan/arc-agi-3-dropped-and-frontier-ai-scored-less-than-1-90cd70e65a61)
- Ndea(Chollet·Knoop, 2025-01 설립; YC W2026): "딥러닝 유도 프로그램 합성"; 2차 보도 기준 $43M, 15명, 제품 없음, 3–5년 연구 런웨이 — [Source](https://x.com/fchollet/status/1879583863368032432); [Source(2차)](https://www.startuphub.ai/ai-news/claudes-corner/2026/claudes-corner-ndea-yc-w2026)
- DreamCoder 후속 Stitch: 라이브러리 학습을 탑다운 합성으로 재정식화, DreamCoder 대비 "3–4자릿수 빠르고 메모리 2자릿수 적음" — [Source](https://arxiv.org/pdf/2211.16605)
- Lake & Baroni MLC(Nature 2023)에 대한 2025 비판 "Fodor and Pylyshyn's Legacy": "여러 사례에서 체계적 행동을 입증하지 못함", 일반화가 "새 원리 발견이 아니라 알려진 규칙의 순열"에 한정 — [Source](https://arxiv.org/html/2506.01820); MLC의 공간추론 확장 Compositional-ARC — [Source](https://arxiv.org/pdf/2504.01445)
- OpenCog Hyperon: PyPI 0.2.10(2026-02-11), 개발 단계 "2 - Pre-Alpha" — [Source](https://pypi.org/project/hyperon/)
- Cyc: 2017년 약 2,450만 공리·150만 용어; 2002년까지 $60M·600인년; Domingos가 "끝없는 데이터 요구와 스스로 진화하지 못함"으로 "파국적 실패"라 평가; Wikipedia(2026-06 갱신)는 Cycorp가 운영 중으로 기재 — [Source](https://en.wikipedia.org/wiki/Cyc). Cyc는 "NLU가 미해결이라 사람 글을 읽어 배울 수 없었고" 자체 언어 CycL만 읽을 수 있었다는 분석 — [Source](https://yuxi-liu-wired.github.io/essays/posts/cyc/)

### Inferences
- "일상적 접지 추론"에 대해 신경기호 축이 **실제로 작동한 형태는 "LLM이 실행 가능한 프로그램/월드모델을 합성하고 환경에서 검증하는 루프"**(SOAR, Tycho, AVO)이며, 순수 기호 KB(Cyc)·하이퍼그래프(OpenCog)는 수십 년째 결과가 없다.
- 소비자 하드웨어 관점: TTT 기반 소형 모델(Qwen-4B급) + 합성 데이터는 RTX 5080에서 돌릴 수 있는 수준이고, CompressARC식 과제별 최적화는 RTX 4070급에서도 가능하다. 그러나 이 능력은 "격자 퍼즐"에 특화되어 있고 사용자 환경 과제로의 전이는 입증되지 않았다.
- Jarvis에 가져갈 가치 있는 패턴은 (1) **정제 루프**, (2) **검증 가능한 프로그램을 도구로 합성**, (3) **테스트타임 미세조정의 합성 데이터 레시피** 이지, 특정 소형 "추론 아키텍처"(HRM/TRM) 자체가 아니다.

### Gaps
- Ndea의 2026년 기술 성과/논문은 1차 출처로 찾지 못했다(투자·인원 정보만).
- ARC-AGI-3 준비공개/비공개셋에서의 2026년 공식 순위는 확인하지 못했다(공개셋 결과만).
- NVARC의 정확한 TTT 하이퍼파라미터·합성 데이터 규모는 2차 요약에 의존.

---

## 5. 지속학습/테스트타임 학습(TTT 레이어, Titans, Nested Learning/HOPE, TTT-E2E, 모듈형 메모리 포지션 페이퍼 2026, NeurIPS 2026 TTCL 워크숍, Pioneer Agent): 파국적 망각의 정직한 평가

### Takeaway
2025-26년의 "가중치 안 테스트타임 학습"(Titans, HOPE, TTT-E2E)은 긴 문맥 압축에서 전망을 보였지만 **Titans는 독립 재구현에서 "기준선을 항상 능가하지 않음"과 재현 불가 수준의 미기재 사항**이 드러났고, 2026년 ICML 스포트라이트 포지션 페이퍼는 "가중치 내 학습(IWL)에만 집중한 지속학습이 파국적 망각을 미해결로 남겼다"며 **문맥 내 학습(ICL) + 모듈형 메모리 + 드문 가중치 통합**을 처방했다. 실무적으로 검증된 지속개선 루프(Pioneer Agent, 2026-04)는 소형 LM의 재학습을 **회귀 제약과 재생(replay) 데이터로 통제**하되, 오케스트레이션은 클라우드 프론티어 모델에 의존한다.

### Cited Findings
- Titans(Google Research, arXiv 2501.00663; NeurIPS 2025): "테스트 시점에 데이터를 파라미터에 저장하는 법을 학습하는 신경 장기기억"; 어텐션=단기, 신경기억=장기; 세 변형 — [Source](https://arxiv.org/pdf/2501.00663); [Source](https://proceedings.neurips.cc/paper_files/paper/2025/file/a4ca07aa108036f80cbb5b82285fd4b1-Paper-Conference.pdf)
- Titans Revisited(arXiv 2510.09551, 독립 재구현): 원논문에 "공개 코드 없음, 설계 선택 미기재(예: 마지막 청크만 예측하는지 전체 청크인지)"; MLM·시계열·추천 과제에서 "**청킹 때문에 Titans가 기준선을 항상 능가하지는 않음**"; 다만 신경기억 구성요소는 어텐션 전용 대비 일관되게 개선 — [Source](https://arxiv.org/abs/2510.09551)
- Nested Learning / HOPE(NeurIPS 2025, "Nested Learning: The Illusion of Deep Learning Architectures"; 2025-11 블로그): 모델을 다층 중첩 최적화 문제로 보고, 자기수정 순환 구조 + "연속체 메모리 시스템(CMS, 갱신 주기가 다른 메모리 모듈 스펙트럼)"; "개념증명(proof-of-concept)" 모델로 언어모델링·지속학습·장문맥에서 개선 주장(논문 내) — [Source](https://research.google/blog/introducing-nested-learning-a-new-ml-paradigm-for-continual-learning/); [Source](https://x.com/GoogleResearch/status/1986855202658418715)
- TTT-E2E(arXiv 2512.23675, Stanford/NVIDIA): 3B 모델·164B 토큰; 8k 슬라이딩 윈도 어텐션 + "문맥을 가중치로 압축"하는 테스트타임 다음토큰 학습; 훈련 시 메타러닝으로 초기화 최적화; 128k 문맥에서 H100 기준 완전 어텐션보다 2.7배 빠름, 문맥 길이와 무관한 상수 지연; Mamba 2·Gated DeltaNet 능가(논문 내) — [Source](https://arxiv.org/abs/2512.23675); 2M 문맥에서 35배 주장은 2차 — [Source(2차)](https://www.deeplearning.ai/the-batch/test-time-training-end-to-end-ttt-e2e-retrains-model-weights-to-handle-long-inputs)
- 포지션: "Modular Memory is the Key to Continual Learning Agents"(arXiv 2603.01761; **ICML 2026 스포트라이트**; Dagstuhl 2025-10 세미나 발): "IWL에만 집중한 역사적 강조가 파국적 망각을 지속적 난제로 남김"; **ICL(일시적 작업기억) + 영속 장기기억으로 사전학습 코어를 조건화하고, 드물게 IWL로 경험을 가중치에 증류**; 파운데이션 모델은 "지속 운영, 경험 축적, 개인화에 근본적으로 제한" — [Source](https://arxiv.org/abs/2603.01761); [Source](https://x.com/mundt_martin/status/2049938326212223327)
- 서베이 "Continual Learning in Transition"(arXiv 2608.06216): 지속학습이 "파라미터 중심 → 시스템 수준 적응"으로 이동; 온폴리시 학습·테스트타임 훈련·"메모리, 스킬 라이브러리, 상호작용 프로토콜 같은 외부 하네스 구성요소"를 포괄하는 3축(언제/어떻게/어디서) 분류 — [Source](https://arxiv.org/abs/2608.06216)
- NeurIPS 2026 TTCL 워크숍(2026-12-12~13, 애틀랜타; JHU·UMich·GaTech·Lambda 조직): TTCL 에이전트 정의 "배치 중 지속적으로 지식·능력을 획득·통합·정제하되 파국적 망각이나 반복적 대규모 재학습 없이"; 현 에이전트는 "새 정보를 내재화하지 못하고, 반복 실수에서 개선되지 못하며, 순진한 갱신 시 망각"; 벤치마크 AgentOdyssey — [Source](https://ttcl-agents.github.io/)
- Pioneer Agent(arXiv 2604.09791, 2026-04): 소형 LM의 콜드스타트·프로덕션 개선 폐루프. 콜드스타트 8개 벤치마크 +1.6~+83.8점; **AdaptFT-Bench 7개 시나리오 전부 유지/개선, 순진한 재학습은 최대 −43점**; 프로덕션 의도분류 84.9→99.3%, 개체 F1 0.345→0.810. 실행은 **Claude Sonnet 4.6(32K thinking, 1M 문맥)이 LangGraph 상태기계로 오케스트레이션**, Modal 샌드박스(16GB), 4–12시간, $13–55(2차 요약) — [Source](https://arxiv.org/abs/2604.09791); [Source](https://arxiv.org/pdf/2604.09791)
- 망각 기전 분석(arXiv 2601.18699, 2026): 2026년 중반 기준 20개 모델(Claude Fable 5, GPT-5.5 High, Gemini 3.5 Flash, DeepSeek-V4-Pro, Llama 4 Maverick, Qwen 3.6-27B 등); "초기층 어텐션 헤드의 엔트로피 분산, 중·심층 FFN/희소 전문가 블록의 국소 표현 붕괴"; LRCP(저랭크 회로 투영)가 "오픈웨이트 설정에서 조상 능력의 최대 94.2% 보존" — [Source](https://arxiv.org/abs/2601.18699). 2차 요약: 헤드의 15–23%가 심각 교란, 망각 정도는 과제 유사도와 r=0.87, **LoRA가 많은 시나리오에서 지식 손실을 막지 못함**, 재생 데이터 5–20% 혼합이 유효 — [Source(2차)](https://arxiv.org/html/2601.18699v2)
- DARPA L2M(2017~): "프로그램 완료" 상태; 과제 영역 CARLA·StarCraft·AI Habitat·AirSim·L2Explorer; 수명 전반의 성능 측정을 위한 "Lifelong Learning Metrics" 정식화(arXiv 2201.08278); 2019년 기준 30개 수행 그룹 — [Source](https://www.darpa.mil/research/programs/lifelong-learning-machines); [Source](https://arxiv.org/abs/2201.08278); [Source](https://www.darpa.mil/news/2019/lifelong-learning-machines)

### Inferences
- 로컬 Jarvis의 지속학습은 **"가중치를 자주 바꾸지 않는" 설계**가 2026년 합의에 가깝다: 경험은 먼저 작업기억/장기기억(벡터·그래프·스킬 라이브러리)에, 가중치 통합은 드물게·재생 데이터와 회귀 테스트를 동반해서. 이는 CPU 노트북에서도 가능한 설계다(메모리는 저장소, 통합은 밤에 배치).
- Titans/HOPE/TTT-E2E는 **처음부터 사전학습해야 하는 아키텍처**(3B·164B 토큰)이므로 개인 하드웨어에서 재현 불가. 다만 TTT-E2E의 "문맥을 가중치로 압축" 아이디어는 소형 어댑터 형태로 실험 가능하다(미검증).
- Pioneer Agent는 "소형 LM 재학습을 자동화할 수 있다"는 증거이지만, 오케스트레이터가 프론티어 클라우드 모델이라 **완전 로컬 지속학습의 증거는 아직 없다**.

### Gaps
- Titans·HOPE의 공식 코드 공개 여부는 재구현 논문이 "없음"이라 했으나 2026년 현재 변화 여부 미확인.
- 망각 분석 논문의 세부 수치(15–23%, r=0.87, LoRA 실패)는 2차 요약 기반이라 원문 대조 필요.
- Pioneer Agent의 모델 크기·로컬 실행 가능성은 초록에 없음.
- "NeurIPS 2026 TTCL" 채택 논문 목록은 2026-10-06 기준 미공개(통지 9/25, 캠레디 10/25).

---

## 6. 생물학적 영감 대안(Thousand Brains Project/Monty, forward-forward, 예측부호화, 뉴로모픽): 실제 시연된 능력

### Takeaway
Monty(TBP)는 **77개 YCB 가정용 물체를 객체당 14개 시점으로 학습해 98.6% 인식·0° 중앙 자세 오차**, 새 회전·잡음 하에서도 73–93%를 유지하는 시뮬레이션 결과를 Neural Computation(2026)에 발표했고, GPU 없이 x86 Linux/macOS에서 돌아가지만 저자 스스로 "초기 단계"이며 합성 객체·실로봇은 미래 과제라고 했다. Forward-forward는 2026년 실데이터 연구에서 ImageNet-100 49.4%(BP >75%)로 "실질적 BP 대체 불가"가 확인되었고, 예측부호화는 2026년에 처음으로 ImageNet 규모(top-5 오류 13.23% vs BP 12.2%)에 도달했다. 뉴로모픽(Loihi 2/Hala Point)은 효율 지표는 인상적이나 Jarvis급 과제 시연은 없다.

### Cited Findings
- TBP 조직: 2025-01-07 Numenta에서 **독립 비영리**로 분리 — [Source](https://discourse.numenta.org/t/announcing-the-independence-of-the-thousand-brains-project/11690). Numenta는 2024-11-20 Monty를 오픈소스로 공개 — [Source](https://www.numenta.com/press/2024/11/20/thousand-brains-project/) (주: 2026-10-06 현재 이 URL은 apicalintelligence.io로 리다이렉트되며 대상 페이지는 404 — 회사명 변경 가능성, 미확인)
- Monty 논문(Neural Computation vol.38 no.6, 2026; arXiv 2507.04494): YCB 77개 물체, 물체당 14개 이미지(정육면체 면·모서리 회전)로 학습; 기본 조건 **분류 98.6%, 중앙 회전오차 0°**; 특징 잡음 95.1%/3°; 새 회전 93.0%/4.5°; 둘 다 88.1%/6°; 균일 색+잡음+새 회전 73.1%/7°. ImageNet-21k 사전학습 ViT와 비교(수치는 본문 4.3절, 발췌 불가). "Monty는 아직 초기(nascent) 단계", 향후 "합성 객체"와 실제 로봇 — [Source](https://arxiv.org/html/2507.04494); [Source](https://doi.org/10.1162/neco.a.1508); 재현 설정 저장소 — [Source](https://github.com/thousandbrainsproject/tbp.tbs_sensorimotor_intelligence)
- Monty 실행 환경: "Windows나 비-x86_64 Linux에서는 작동하지 않음"(WSL 가이드 별도), conda + Habitat-sim(YCB 렌더링), Apple Silicon은 Rosetta 2; **GPU 요구 없음** — [Source](https://docs.thousandbrains.org/docs/getting-started)
- 2026-05 포럼 "Monty's Capabilities Roadmap and Open Theoretical Questions": 역량별 진행도와 미해결 이론 문제를 Excalidraw/영상으로 공개(텍스트 요약 없음) — [Source](https://forum.thousandbrains.org/t/2026-05-montys-capabilities-roadmap-and-open-theoretical-questions/1150)
- Numenta 회사 자체는 2023-09 NuPIC(CPU LLM 추론 가속) 상용 제품으로 전환, 2024-05 NuPIC 2.0; Intel AMX 기반 — [Source](https://www.businesswire.com/news/home/20230911597897/en/Numenta-Transforms-the-AI-Landscape-with-NuPIC); [Source](https://www.businesswire.com/news/home/20240521909416/en/Numenta-Unveils-NuPIC-2.0-Elevates-CPUs-to-Superior-Choice-for-Running-AI-Models)
- Forward-forward 실데이터 한계(arXiv 2606.06539, 2026): 합성 K-스윕이 "출력 차원과 세밀 판별 난이도를 혼동해 FF 전이성을 과대평가"; CIFAR-10 −2.40pp, CIFAR-100 −5.93pp, **ImageNet-100(224px) 49.4% vs BP >75%**; 메모리 효율 주장도 불성립(FF 7.90GB vs BP+그래디언트 누적 4.18GB); "층-국소 학습은 의미 있는 규모에서 BP 대안이 되지 못하고 있음" — [Source](https://arxiv.org/abs/2606.06539). 2025년 FF 개선 계보(ASGE 등) — [Source](https://arxiv.org/pdf/2509.12394)
- 예측부호화 ImageNet(arXiv 2606.03584, 2026): 10층 합성곱 PCN(VGG10)을 평형전파(EP)로 학습, **top-5 오류 13.23% vs BP 12.2%**, "PCN과 EP 모두 ImageNet 규모 첫 시연" — [Source](https://arxiv.org/abs/2606.03584). 깊은 PCN(100층 이상) 안정 학습, 13층 17.26M PCN이 ImageNet에서 ResNet-34 능가·ResNet-50 근접(2차 요약) — [Source](https://arxiv.org/pdf/2510.23323)
- 뉴로모픽: Hala Point = Loihi 2 1,152개, 11.5억 뉴런·1,280억 시냅스, 20 petaops, 15 TOPS/W(배치 없이) — [Source](https://newsroom.intel.com/artificial-intelligence/intel-builds-worlds-largest-neuromorphic-system-to-enable-more-sustainable-ai); Loihi 2에서 LLM 추론: 프리필 2배 처리량·토큰당 에너지 약 1/2, 생성 시 약 3배 처리량(논문 내) — [Source](https://arxiv.org/html/2503.18002v2); 2026-05 기준 Intel 공식 하드웨어는 여전히 Loihi 2/Hala Point(2차) — [Source(2차)](https://semiconductorinsight.com/blog/loihi-2-and-hala-point-2026-update-how-neuromorphic-ic/)

### Inferences
- Monty는 CPU 전용 노트북에서 돌아가는 **유일한 "처음부터 학습하는 감각운동" 프레임워크**이지만, 현재 입증 범위는 "시뮬레이션 3D 물체 인식/자세"이며 언어·화면·사용자 모델링은 없다. Jarvis의 "물리 환경 객체 모델" 모듈 후보일 뿐 중추가 될 수 없다.
- FF/PC/뉴로모픽은 **학습 알고리즘/하드웨어 대안**이지 에이전트 아키텍처가 아니며, 2026년 기준 BP+GPU 대비 성능 우위를 보이지 못했다. 로컬 Jarvis 설계 결정에서 제외해도 무방하다.

### Gaps
- Monty vs ViT 비교 수치, 추론 시간, FLOP 추정치는 본문 열람 필요.
- 2026 로드맵의 실제 역량 진행도(영상/다이어그램)는 텍스트로 확인 불가.
- numenta.com → apicalintelligence.io 리다이렉트의 의미(사명 변경 여부)는 확인 실패.
- Loihi 3 관련 2026년 자료는 2차 블로그뿐이라 제외.

---

## 7. 2026년 소비자 하드웨어 소형 모델 에이전트(1B–14B 로컬 모델, 로컬 음성 스택, OpenJarvis/Leon/Jarvis CLS): 무엇이 입증되었나

### Takeaway
2026년 10월 기준 가장 많이 **독립적으로 측정된** 결과는 "소형 로컬 LM이 **적절히 라우팅하면 단일턴 채팅·추론 질의의 88.7%** 를 처리"(Stanford IPW, 2025-11)하고, OpenJarvis(2026-05)가 **Qwen3.5-9B 온디바이스로 클라우드(Claude Opus 4.6) 대비 평균 3.2pp 이내** 격차를 낸다는 것이다. 단, 같은 논문은 **클라우드를 로컬로 단순 치환하면 개인 AI 과제에서 25–39pp 하락**한다고 밝혔고, 격차를 줄인 "스펙 탐색"은 탐색 단계에서 프론티어 클라우드 모델을 쓴다. Gemma 4 E4B(오디오 입력 내장, Apache 2.0)와 Qwen3.5 0.8B–9B(Apache 2.0)가 RTX 5080 16GB는 물론 노트북 CPU 급의 언어 기관 후보이며, faster-whisper+Silero VAD+Kokoro(또는 Moshi/Pocket TTS) 로컬 음성 스택이 성숙했다. "Jarvis CLS"는 식별 가능한 1차 출처를 찾지 못했다.

### Cited Findings
- Intelligence per Watt(arXiv 2511.07885, 2025-11; Stanford·Together, Ré·Hennessy 공저): 20개+ 로컬 LM, 8개 가속기, 100만 실사용 질의; 2025-10 기준 **라우팅 시 88.7%** 질의 처리 가능, 단일 모델은 23.2%(2023)→71.3%(2025); IPW 2년간 5.3배(모델 3.1배 × 가속기 1.7배) — [Source](https://arxiv.org/html/2511.07885v1); [Source](https://www.alphaxiv.org/abs/2511.07885)
- OpenJarvis(arXiv 2605.17172, 2026-05-16; Stanford Scaling Intelligence/Hazy Research + Lambda; 프레임워크 공개 2026-03-12, Apache 2.0): 5개 프리미티브(Intelligence, Engine, Agents, Tools & Memory, Learning)의 타입 명세; "LLM 유도 스펙 탐색"(탐색 시 프론티어 클라우드 모델이 편집 제안, 비회귀 수정만 채택, 추론은 온디바이스); 로컬-클라우드 정확도 격차 평균 3.2pp, 한계 API 비용 약 1/800, 지연 1/4; 8개 벤치마크 중 4개에서 클라우드 동등/상회; **단순 치환 시 PinchBench·GAIA 등에서 25–39pp 하락**; 비교 모델 Qwen3.5-9B vs Claude Opus 4.6 — [Source](https://arxiv.org/abs/2605.17172); [Source](https://github.com/open-jarvis/OpenJarvis); [Source(2차)](https://www.marktechpost.com/2026/03/12/stanford-researchers-release-openjarvis-a-local-first-framework-for-building-on-device-personal-ai-agents-with-tools-memory-and-learning/)
- Qwen3.5 소형(2026-03-02): 0.8B/2B/4B/9B, Apache 2.0, 조기융합 멀티모달; 9B·4B는 "표준 노트북", 0.8B·2B는 스마트폰 대상; 4B 네이티브 262,144 토큰 문맥; 9B MMMU-Pro 70.1, Video-MME 84.5; 4B Video-MME 83.5, HMMT 74.0 — [Source(2차)](https://venturebeat.com/technology/alibabas-small-open-source-qwen3-5-9b-beats-openais-gpt-oss-120b-and-can-run). **GPQA Diamond 수치 상충**: VentureBeat 81.7 vs 다른 2차 출처 52.1 — [Source(2차)](https://www.digitalapplied.com/blog/qwen-3-5-small-models-9b-beats-gpt-phone-guide) (원 모델카드 미확인)
- Gemma 4(2026-04-02, Apache 2.0): E2B(유효 2.3B/임베딩 포함 5.1B), E4B(4.5B/8B), 12B Unified, 26B A4B MoE, 31B dense; 128k–256k 문맥; E2B/E4B에 USM형 컨포머 **오디오 입력(최대 30초)**; E4B MMLU-Pro 69.4(Gemma 3n E4B 50.6 대비 +18.8), AIME 2026 42.5, LiveCodeBench 52.0; 26B A4B MMLU-Pro 82.6; llama.cpp/MLX/WebGPU 지원 — [Source](https://huggingface.co/blog/gemma4); [Source](https://huggingface.co/google/gemma-4-E4B); 기술보고서 — [Source](https://arxiv.org/pdf/2607.02770)
- Qwen3.8(2026-08): 27B 오픈웨이트 공개, 2.4T MoE 플래그십, Flash-Next 125B-A6B(2026-08-26) — [Source](https://github.com/QwenLM/Qwen3.8); [Source(2차)](https://www.yottalabs.ai/post/qwen-3-8-27b-specs-hardware-requirements-how-to-run-2026)
- RTX 5080 16GB 적합성(2차 벤치 사이트): 14B Q5_K_M(12.9GB)이 최적, 14B 약 58 tok/s; Mistral Small 3.1 24B Q4_K_M 약 13GB·55 tok/s; gpt-oss 20B Q4 약 40 tok/s; **Qwen 3.6 27B Q4는 17–18GB로 미적합**; 대역폭 960GB/s, 제약은 VRAM — [Source(2차)](https://turbollm.dev/gpu/rtx-5080); [Source(2차)](https://modelfit.io/gpu/rtx-5080/); [Source(2차)](https://toolhalla.ai/blog/best-local-llms-rtx-5080-2026)
- 8GB급/노트북(2차): 8B Q4_K_M ≈ 6GB VRAM, RTX 3080에서 35–45 tok/s — [Source(2차)](https://www.sitepoint.com/best-local-llm-models-2026/); M1 MacBook Air에서 Qwen3.5 소형 로컬 실행(개발자 증언, VentureBeat 인용)
- 로컬 음성 스택(2026): Silero VAD(턴 감지) → faster-whisper(STT) → Ollama(LLM 스트리밍) → Kokoro(TTS, 첫 문장 즉시 합성); 전부 허용 라이선스·오프라인 — [Source(2차)](https://innerzero.com/blog/best-open-source-local-ai-voice-assistant-jarvis-2026); 구현 예 — [Source](https://github.com/ShayneP/local-voice-ai)
- 전이중(full-duplex) 음성: Moshi(Kyutai) 이론 지연 160ms(Mimi 80ms + 음향 80ms), L4 GPU에서 실측 약 200ms, STT/LLM/TTS 파이프라인 없이 음성 토큰 직접 처리 — [Source](https://kyutai.org/Moshi.pdf); [Source](https://github.com/kyutai-labs/moshi); Kyutai Pocket TTS(2026-01)는 CPU 실시간(2차) — [Source(2차)](https://www.spheron.network/blog/speech-to-speech-gpu-cloud-moshi-sesame-csm-hertz-dev/)
- Leon(2017~, MIT, 17k 스타): 2.0 Developer Preview에서 "의도 분류 → 완전 에이전트 구조(계층 메모리, 네이티브 스킬, SKILL.md 워크플로, **제한된 선제적 펄스(bounded proactive pulse) 시스템**)"로 재설계; 최근 12개월 706 커밋 — [Source](https://github.com/leon-ai/leon); [Source(2차)](https://ideaproof.io/open-source/project/leon); [Source(2차)](https://www.promptquorum.com/power-local-llm/leon-ai-review)
- "Jarvis CLS": 검색에서 해당 이름의 시스템을 찾지 못함. 인접 자료로 JARVIS-1(메모리 증강 멀티모달 LM 마인크래프트 에이전트), CLS 이론 기반 지속학습(CLS-ER 등), 2026 에이전트 메모리 서베이("Memory in the Age of AI Agents", "Always-On Agents")만 확인 — [Source](https://www.researchgate.net/publication/386487236_JARVIS-1_Open-World_Multi-Task_Agents_With_Memory-Augmented_Multimodal_Language_Models); [Source](https://arxiv.org/abs/2201.12604); [Source](https://arxiv.org/pdf/2512.13564); [Source](https://arxiv.org/pdf/2606.30306)

### Inferences
- "언어/추론 기관"으로서 2026년 로컬 소형 모델은 **단일턴 질의의 대부분**을 감당하지만, **개인 AI 과제(도구 사용·장기 과제)에서는 하네스 최적화 없이는 25–39pp 열세**다. 즉 소형 모델 자체보다 **하네스(메모리·도구·학습 루프) 설계가 성능을 좌우**한다는 것이 OpenJarvis의 실증 메시지다.
- 하드웨어 매핑: RTX 5080 16GB → Qwen3.5-9B(Q8) 또는 Gemma 4 26B A4B(Q4, 활성 4B라 속도 양호; VRAM 적합성은 미확인) + V-JEPA 2 ViT-L 인코더 + 15M 잠재 월드모델 동시 상주 가능. CPU 노트북 → Gemma 4 E2B/E4B 또는 Qwen3.5 2B/4B + Pocket TTS + faster-whisper small.

### Gaps
- Qwen3.5-9B GPQA Diamond(81.7 vs 52.1) 상충을 모델카드로 해소하지 못함.
- Gemma 4 26B A4B의 16GB 적합 여부(양자화별 VRAM) 1차 출처 없음.
- RTX 5080 토큰/초 수치는 모두 2차 벤치 사이트.
- OpenJarvis의 하드웨어 테스트 목록, 학습 루프가 미세조정인지 프롬프트/스펙 최적화인지(초록은 "스펙 탐색"만 언급) 본문 확인 필요.
- "Jarvis CLS"는 미확인 — 사용자가 뜻한 대상이 무엇인지 재확인 필요.

---

## 8. 정직한 실패 분석(Cog/Kismet, Cyc, OpenCog, Numenta HTM, Vicarious, DARPA L2M)과 반복되는 교훈

### Takeaway
비스케일링 프로그램의 실패/정체는 (1) **하위 시스템 통합의 "일관된 전역 행동" 부재**(Cog), (2) **손으로 설계한 사전지식/지식의 비확장성**(Cyc, AXIOM 저자 인정), (3) **평가 규약·과제 조건화에 숨은 성능**(HRM/TRM/LeWM/Titans 재현), (4) **상업 압력에 따른 피벗**(Numenta→NuPIC, Vicarious→Intrinsic/DeepMind, VERSES→Genius), (5) **시뮬레이션 지표 체계는 만들었지만 실세계 지속학습 시연 없이 종료**(DARPA L2M) 라는 다섯 패턴으로 반복된다.

### Cited Findings
- Cog(MIT, Brooks): "기존 하위 시스템·하위 행동들로부터 일관된 전역 행동을 시연하는 것이 빠진 핵심 조각"; 액추에이터 경쟁과 "세계를 통한 의도치 않은 결합"이 간섭 유발 — [Source](https://people.csail.mit.edu/brooks/papers/CMAA-group.pdf); Kismet/Cog 사용자 연구 중 시선 추적·청각·발성 장애, Cog 한쪽 팔 고장 — [Source](https://web.mit.edu/people/sturkle/encounterswithkismet.pdf)
- Cyc: 2002년까지 $60M·600인년, 2017년 2,450만 공리; "끝없는 데이터 요구와 자체 진화 불가"(Domingos); 독점적 구조가 학계 검증을 막음; NLU 미해결로 "사람 글을 읽어 배울 수 없음" — [Source](https://en.wikipedia.org/wiki/Cyc); [Source](https://yuxi-liu-wired.github.io/essays/posts/cyc/). Lenat 2023-09-23 사망; "Cyc 종료" 기사(hyper.ai)는 구체 근거 없음 — [Source(2차, 미검증)](https://hyper.ai/en/stories/7fbccbd9df41bdccaf0907a1d60cc7d5)
- OpenCog: 2008년대 OpenCog Classic 이후 Hyperon(MeTTa) 재설계, 2026-02 현재 Pre-Alpha — [Source](https://pypi.org/project/hyperon/); [Source](https://arxiv.org/html/2310.18318)
- Numenta/HTM: HTM은 "구형 알고리즘 구현", 이론은 Thousand Brains로 전환 — [Source](https://www.numenta.com/resources/htm/); 회사는 NuPIC(CPU LLM 추론)로 상용 피벗(2023–24) — [Source](https://www.businesswire.com/news/home/20230911597897/en/Numenta-Transforms-the-AI-Landscape-with-NuPIC); 연구 부문은 2025-01 TBP 비영리로 분리 — [Source](https://discourse.numenta.org/t/announcing-the-independence-of-the-thousand-brains-project/11690)
- Vicarious(2010 설립, $200M+ 조달): 2022-04-22 Alphabet Intrinsic에 인수, AGI 연구팀(Dileep George)은 DeepMind로 — [Source](https://techcrunch.com/2022/04/22/alphabet-owned-intrinsic-is-acquiring-fellow-robotic-software-firm-vicarious/); [Source](https://en.wikipedia.org/wiki/Vicarious_(company))
- DARPA L2M: "프로그램 완료"; 산출물은 시뮬레이션 도메인(CARLA, StarCraft, Habitat, AirSim, L2Explorer)과 평가 지표 체계 — [Source](https://www.darpa.mil/research/programs/lifelong-learning-machines); [Source](https://arxiv.org/abs/2201.08278); [Source](https://arxiv.org/pdf/2203.07454)
- 2025-26 재현 실패/과대평가 사례(앞 절 참조): HRM 41%→검증 32%, 성능이 평가 과제 학습에서 기인 — [Source](https://arcprize.org/blog/hrm-analysis); TRM 퍼즐 ID 제거 시 0% — [Source](https://arxiv.org/abs/2512.11847); LeWM 평가 규약에 따라 84%→8% — [Source](https://arxiv.org/abs/2608.10145); Titans 코드 부재·기준선 미능가 — [Source](https://arxiv.org/abs/2510.09551); FF 합성 벤치 과대평가 — [Source](https://arxiv.org/abs/2606.06539); AXIOM "사전지식은 설계된 것" — [Source](https://arxiv.org/html/2505.24784v1)
- VERSES의 벤치마크는 자체 설계(Gameworld 10k, LLM 보조 생성) — [Source](https://arxiv.org/html/2505.24784v1)

### Inferences
- 반복 교훈 → 설계 원칙: (a) **통합(integration)이 알고리즘보다 어렵다** → 처음부터 단일 조정 루프(하나의 메모리, 하나의 행동 선택기) 위에 모듈을 얹을 것; (b) **손설계 사전지식은 커버리지를 못 늘린다** → 사전지식은 사전학습된 오픈 모델(V-JEPA/소형 LM)에서 가져오고, 로컬에서는 "무엇을 기억·통합할지"만 학습; (c) **평가 규약을 먼저 고정**(사용자 환경 과제 세트와 회귀 테스트)하지 않으면 자기 기만에 빠진다; (d) 상업 피벗 위험은 개인 프로젝트에는 해당 없지만, **의존하는 외부 프로젝트의 지속성**(라이선스·유지보수)을 선택 기준에 넣어야 한다.

### Gaps
- Cycorp의 2025-26년 운영 상태를 1차 출처로 확인하지 못함.
- DARPA L2M의 "교훈(lessons learned)" 공식 문서는 찾지 못했다(지표 논문과 시뮬 도메인 정보만).
- Vicarious의 RCN 등 기술 자산이 DeepMind에서 어떻게 쓰였는지는 불명.

---

## 9. 종합: 로컬·지속학습·선제적 Jarvis에 가장 높은 정직한 성공 확률을 가진 단일 경로

### Takeaway
조사된 비스케일링 계열 중 **"사용자 환경 이해 + 선제 행동 + 지속학습"을 하나라도 로컬에서 시연한 순수 비-LM 경로는 없다**. 2026년 10월 기준 가장 정직한 선택은 **"소형 오픈 LM(4–9B)을 언어·계획 기관으로 쓰는 모듈형 메모리 하네스(OpenJarvis/Leon 패턴) + ICL 우선·드문 가중치 통합의 지속학습 + 학습진전(LP) 기반 선제적 탐색 모듈 + (선택) 15M급 잠재 월드모델로 화면/센서 동역학 예측"** 이며, 이것은 "스케일업된 자기회귀 LM에 의존하지 않는다"는 제약을 **4–9B 로컬 모델 범위에서** 만족한다. JEPA/능동추론/TBP는 이 하네스 안의 **교체 가능한 감각 모듈** 로만 들어가야 한다.

### Cited Findings
- 로컬 소형 모델의 커버리지(88.7%)와 하네스 최적화 시 클라우드 대비 3.2pp 격차, 단순 치환 시 25–39pp 하락 — [Source](https://arxiv.org/html/2511.07885v1); [Source](https://arxiv.org/abs/2605.17172)
- 지속학습 처방(ICL+모듈 메모리+드문 IWL), ICML 2026 스포트라이트 — [Source](https://arxiv.org/abs/2603.01761); 재학습 시 회귀 통제가 없으면 최대 −43점 — [Source](https://arxiv.org/abs/2604.09791)
- LP 기반 목표 선택이 LLM 에이전트에서 작동(MAGELLAN) — [Source](https://arxiv.org/abs/2502.07709)
- 15M 잠재 월드모델을 단일 GPU 수 시간에 학습 가능하나 평가 규약 민감 — [Source](https://le-wm.github.io/); [Source](https://arxiv.org/abs/2608.10145)
- 순수 비-LM 경로의 현재 입증 범위: AXIOM=단색 스프라이트 게임(저자 인정 확장 불가) — [Source](https://arxiv.org/html/2505.24784v1); Monty=시뮬 YCB 물체 인식("초기 단계") — [Source](https://arxiv.org/html/2507.04494); V-JEPA 2-AC=시각 목표 pick-place, 행동당 16초 — [Source](https://arxiv.org/html/2506.09985); BabyBench=자기접촉/손 응시 — [Source](https://babybench.github.io/2025/)
- ARC-AGI-3의 상호작용 과제를 100% 푼 시스템은 모두 LLM 코딩 에이전트 + 실행 가능한 프로그램형 월드모델 — [Source](https://arxiv.org/abs/2607.28287); [Source](https://developer.nvidia.com/blog/nvidia-avo-reaches-100-on-arc-agi-3-demonstrating-a-frontier-level-general-purpose-architecture-for-long-horizon-autonomous-agents/)
- 소비자 하드웨어 적합 모델과 음성 스택 — [Source](https://huggingface.co/blog/gemma4); [Source(2차)](https://venturebeat.com/technology/alibabas-small-open-source-qwen3-5-9b-beats-openais-gpt-oss-120b-and-can-run); [Source(2차)](https://turbollm.dev/gpu/rtx-5080); [Source](https://kyutai.org/Moshi.pdf)

### Inferences
- **경로별 정직한 확률 평가(조사 근거 기반 추정, 수치는 주관)**:
  - (A) 소형 LM + 모듈형 메모리 하네스 + LP 탐색: 입증된 구성요소만 조합 → 1년 내 "환경을 이해하고 선제적으로 돕는" 최소 Jarvis 가능성 가장 높음. 리스크는 "지속학습이 사실상 메모리 축적에 그침"과 소형 모델의 도구 사용 열세(하네스로 보완).
  - (B) JEPA 잠재 월드모델 중심(LeWM/V-JEPA 2 기반, 언어 없이): 화면/센서 예측은 가능하나 의도·언어·행동 접지가 없고 재현 민감성이 큼 → 중추로는 낮음, 모듈로는 유망.
  - (C) 능동추론(AXIOM형): 소형·빠름이지만 객체 사전분할 전제와 학술 라이선스 → 낮음.
  - (D) 프로그램 합성/TTT(ARC형): 퍼즐 특화, 일상 과제 전이 미입증 → 낮음(단, "검증 가능한 프로그램 합성" 패턴은 A에 포함).
  - (E) 가중치 내 TTT 아키텍처(Titans/HOPE/TTT-E2E): 사전학습 필요·재현성 문제 → 개인 하드웨어에서 불가.
  - (F) TBP/Monty: CPU 가능하나 입증 범위 협소, 언어 없음 → 낮음(물체 모듈 후보).
  - (G) FF/PC/뉴로모픽: 아키텍처가 아님 → 제외.
- 하드웨어 계층화: RTX 5080(Qwen3.5-9B Q8 또는 Gemma 4 26B A4B + V-JEPA 2 ViT-L + 15M 잠재 예측기 + Kokoro/Moshi) ↔ CPU 노트북(Gemma 4 E2B/E4B 또는 Qwen3.5 2B/4B + faster-whisper small + Pocket TTS; 월드모델·통합은 야간 배치 또는 생략)로 **동일 하네스를 모델만 바꿔** 운용하는 것이 2026년 결과에 부합한다.
- 성공 조건으로 조사가 시사하는 세 가지 선행 작업: (1) 사용자 환경 과제 세트와 회귀 테스트를 먼저 고정(HRM/TRM/LeWM 교훈), (2) 메모리→통합 파이프라인에 재생 데이터·회귀 제약 내장(Pioneer/망각 연구), (3) 선제 행동의 범위를 Leon식 "bounded proactive pulse"처럼 제한해 안전·검증 가능하게 설계.

### Gaps
- 위 경로 (A)를 **완전 로컬로 장기간 운용하며 지속학습을 정량 평가한 공개 사례**는 찾지 못했다(OpenJarvis는 스펙 탐색에 클라우드 사용, Pioneer는 클라우드 오케스트레이션).
- "선제적(proactive) 행동"을 평가하는 공인 벤치마크는 확인하지 못했다(AgentOdyssey는 텍스트 게임).
- 소형 LM + LP 탐색(MAGELLAN형)을 실제 데스크톱 환경에 적용한 결과는 없다.
