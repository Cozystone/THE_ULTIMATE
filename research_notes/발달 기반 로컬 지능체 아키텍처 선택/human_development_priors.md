# 인간 영아의 선천적 자질(innate priors)과 인지 발달의 전개 — "기계 유아(machine infant)" 설계를 위한 참조 노트

작성일: 2026-10-06. 범위: (a) 신생아의 선천적 자질, (b) 발달 이정표의 순서·시기, (c) 발달을 구동하는 기제, (d) 인공 발달 에이전트에서 하드코딩 vs 학습 후보. 확립된 발견과 논쟁 중인 주장을 구분하고, 각 핵심 발견에 발표 연도를 병기했다. 웹 검색 예산 소진으로 일부 1차 문헌은 초록·요약 수준에서만 확인했으며 해당 항목은 각 절의 Gaps에 명시했다.

---

## 1. Spelke의 핵심 지식(core knowledge) 이론: 무엇이 선천적이라 주장되며 증거는 무엇인가, 비판자들은 무엇을 말하는가

### Takeaway
Spelke 진영은 대상(objects)·행위자(agents)·수(number)·공간/기하(space)·(아마도) 사회적 파트너의 4~5개 영역특정 핵심 시스템이 선천적이라 주장하며, 기대 위반(violation-of-expectation) 응시시간 연구·신생아 연구·병아리 비교연구가 주된 증거다. 비판자들은 "그 시스템이 어떻게 발생하는가"라는 기제 설명이 비어 있다는 점(발달 시스템/역동 시스템), 응시시간 해석의 과잉, 그리고 2024년 이후 단일 아동 헤드캠 데이터로 학습한 범용 신경망이 강한 귀납 편향 없이도 상당한 시각·어휘 표상을 획득한다는 결과(Orhan & Lake, Vong et al.)를 들어 "강한 선천 구조가 필수"라는 추론을 약화시킨다. 다만 후자의 연구들 자체도 "무엇이 선천적이어야 하는지는 열린 경험적 질문"이라고 명시한다.

### Cited Findings
- Spelke & Kinzler(2007)는 인간 인지가 대상, 행위(행위자의 목표 지향 행동), 수(집합의 순서·덧셈·뺄셈 관계), 공간(장소의 기하적 배치)의 네 핵심 시스템 위에 세워지며, 사회적 파트너를 표상하는 다섯 번째 시스템이 있을 수 있다고 주장했다. 각 시스템은 계통발생·개체발생에 깊은 뿌리를 가지며 영역특정적이다 — [Spelke & Kinzler 2007, Developmental Science](https://onlinelibrary.wiley.com/doi/10.1111/j.1467-7687.2007.00569.x); [PDF](https://www.harvardlds.org/wp-content/uploads/2017/01/SpelkeKinzler07-1.pdf)
- Spelke의 *What Babies Know*(2022)는 두 가설을 옹호한다: (1) 인간과 다른 종은 추상 개념으로 환경을 표상하는 선천적 계산 구조(핵심 지식 시스템)를 갖는다, (2) 인간만이 자연언어를 습득하며, 그 통사와 합성 의미론이 핵심 시스템 출력을 결합해 새 개념을 구성하게 한다 — [Revencu & Csibra 2023 서평 요약, CEU](https://research.ceu.edu/en/publications/the-missing-link-between-core-knowledge-and-language-review-of-el/); [Mind & Language](https://onlinelibrary.wiley.com/doi/10.1111/mila.12482)
- Revencu & Csibra(2023)는 가설 (1)은 수용하지만, 언어 습득만으로 인간 인지의 생산성이 설명된다는 점을 의심하고, 특히 "영아가 언어의 측면을 이용해 타인에 대한 새로운 개념을 발달시킨다"는 주장에 반대한다 — [CEU](https://research.ceu.edu/en/publications/the-missing-link-between-core-knowledge-and-language-review-of-el/)
- Moore & Lewkowicz(2023)는 이 책이 핵심 지식 이론의 학술적 제시이자 지지 증거의 방대한 집대성이지만, "핵심 지식은 단순히 인지 진화의 선천적 산물"이라는 가정이 해당 인지 시스템의 출현을 뒷받침하는 발달 기제를 설명하지 못한다고 비판하며 발달 시스템/후성(epigenetic) 관점을 대안으로 제시한다 — [Moore & Lewkowicz](https://scholarship.claremont.edu/pitzer_fac_pub/220/)
- BBS에 실린 25편의 논평은 지식의 기원, 도상적(iconic) 대 명제적 표상의 상호작용, 수·사회 인지의 구조, 인간 고유 능력의 원천, 핵심 지식·지각·사고의 경계에 관한 질문을 제기했고 Spelke가 답했다(2024) — [PhilPapers: Spelke, Response to commentaries](https://philpapers.org/rec/SPERTC-6); [Scholl 논평 PDF](https://perception.yale.edu/papers/24-Scholl-BBSComment.pdf)
- 증거 유형 1(기대 위반): Baillargeon의 연구에서 생후 3.5개월 영아가 "불가능한 사건"(숨겨진 쥐를 통과해 지나가는 자동차 등)에 놀람(응시 증가)을 보였으며, 이는 Piaget가 상정한 대상영속성 시기(감각운동기 말, 약 18~24개월)보다 훨씬 이르다 — [Wikipedia: Object permanence](https://en.wikipedia.org/wiki/Object_permanence)
- 증거 유형 2(신생아): Izard et al.(2009, PNAS)은 신생아가 4~18개 대상의 정지 시각 배열을 청각 사건 시퀀스와 수(number)를 기준으로 자발적으로 연합함을 보였다 — [PNAS 2009](https://www.pnas.org/doi/10.1073/pnas.0812142106)
- 증거 유형 3(비교/통제 사육): 갓 부화한 병아리는 시각 경험 시작 시점에 이미 불변 대상 표상을 생성하며(Wood 2013, PNAS), Vallortigara 그룹은 병아리를 모델로 대상·수·기하의 핵심 지식을 연구한다 — [PNAS 2013](https://www.pnas.org/doi/abs/10.1073/pnas.1308246110); [Core knowledge as a neuro-ethologist views it (2024)](https://www.researchgate.net/publication/381771558_Core_knowledge_as_a_neuro-ethologist_views_it); [병아리 대상영속성은 반대 증거에도 견고함(2024 preprint)](https://arxiv.org/pdf/2402.14641)
- 증거 유형 4(태아): Reid et al.(2017)은 임신 3분기 태아가 자궁벽을 통해 투사된 얼굴형 배열(역삼각형 3점)을 역전 배열보다 더 많이 머리를 돌려 쫓는다고 보고했으나, 방법론적 비판이 제기되었다 — [Current Biology 2017](https://www.cell.com/current-biology/comments/S0960-9822(17)30580-8); [방법론 비판, Current Biology 2018](https://www.sciencedirect.com/science/article/pii/S0960982218303762)
- 구성주의/역동 시스템 비판: Smith & Thelen은 Piaget의 A-not-B 오류를 "기억된 위치로의 목표 지향 도달(reaching)을 만드는 일반 과정"으로 설명했다. 기억 흔적 강도·표적 현저성·대기 시간·자세가 결합해 수행을 결정하며, B 시행에서 앉은 자세를 선 자세로 바꾸기만 해도 10개월 영아가 12개월처럼 올바르게 탐색했다(A 탐색 경험의 현저성 감소) — [Wikipedia: A-not-B error](https://en.wikipedia.org/wiki/A-not-B_error); [Smith & Thelen 2003 TiCS PDF](http://wexler.free.fr/library/files/thelen%20(2003)%20development%20as%20a%20dynamic%20system.pdf); [Child Development 2001 PDF](https://dynamicfieldtheory.org/upload/file/1553696600_2c38d099fc0dbd9d90a3/cdev351.pdf)
- 신경구성주의 비판: Karmiloff-Smith(*Beyond Modularity*, 1992)는 마음이 사전 명세된 모듈로 시작하지 않고, 발달 과정에서 점진적 모듈화(modularization)와 표상 재기술(representational redescription; 암묵 지식의 점진적 명시화)이라는 두 병행 과정을 거친다고 주장했다(Fodor 선천주의와 Piaget 구성주의의 종합 시도) — [PhilPapers: Précis of Beyond Modularity](https://philpapers.org/rec/KARPOB); [Birkbeck 서론 장](https://psyc.bbk.ac.uk/dnl/wp-content/uploads/sites/10/2025/07/AKS_ThinkingDevelopmentally_CH1.pdf)
- 딥러닝 기반 반론(Orhan & Lake 2024, Nature Machine Intelligence): SAYCam 3명 아동(6~31개월, 각 194/141/137시간, 총 472시간, 5fps 약 900만 프레임)의 헤드캠 영상만으로 자기지도 학습(DINO/Mugs/MAE, ViT·ResNeXt)한 모델이 하류 과제에서 ImageNet 학습 모델의 65~70% 성능(ImageNet 10%로 학습한 모델과 유사; 무작위 기준 18.6% 대비 SAY 70.2%)을 보였고, 개·새·파충류·곤충·차량 같은 넓은 의미 범주와 객체 위치화를 지도 없이 학습했다. 반면 ImageNet 전체 학습 모델보다 덜 객체중심적이고, 작은 객체 위치화와 세부 묘사에 약했다. 저자들은 "분류학적 일반화 편향 같은 강한 귀납 편향은 불필요할 수 있다"면서도 "인간 수준 이해에 더 강한 객체중심 편향이 필요한지는 열린 경험적 질문"이라 했다. 한계로 (i) 약 40일치 경험 대 아동은 두 자릿수 더 많은 경험, (ii) 시각 단일 모달, (iii) 경사하강의 생물학적 비현실성, (iv) 수동 학습 vs 상호작용 학습을 명시했다 — [arXiv HTML](https://arxiv.org/html/2305.15372); [Nature MI 2024](https://www.nature.com/articles/s42256-024-00802-0)
- Vong et al.(2024, Science): 한 아동(6~25개월)의 61시간 시각-언어 상관 데이터로 "상대적으로 범용적인" 대조 신경망(CVCL)을 학습시켜, 아동 일상 속 다수의 단어-지시체 매핑을 획득하고 새 시각 지시체로 제로샷 일반화하며 시각·언어 개념 체계가 정렬됨을 보였다. 저자들은 "초기 단어 의미의 핵심 측면은 한 아동의 입력에서 공동 표상·연합 학습으로 학습 가능"하다고 결론했다 — [Science 2024 (DOI)](https://doi.org/10.1126/science.adi1374)
- Vong & Lake(2025)는 위 결과가 SAYCam 세 아동 모두와 복수 모델 아키텍처에서 견고함을 확인했다 — [Open Mind 2025](https://direct.mit.edu/opmi/article/doi/10.1162/OPMI.a.377/138563/On-the-Robustness-of-Modeling-Grounded-Word)
- 선천 편향 옹호(AI 쪽): Lake, Ullman, Tenenbaum & Gershman(2017, BBS)은 발달 초기의 "start-up software"로 직관 물리(대상의 지속·고체성·응집성, 물리적으로 불가능한 궤적 배제)와 직관 심리(타인의 목표·신념)를 들고, 기계가 인과 모델·직관 이론·합성성·학습의 학습(learning-to-learn)을 갖춰야 한다고 주장했다 — [BBS 2017 PDF](https://www.cs.princeton.edu/~bl8144/papers/LakeEtAl2017BBS.pdf)
- Zador(2019)의 "순수 학습 비판": 동물 행동의 상당 부분은 학습이 아니라 유전적으로 부호화된 프로그램이며, 게놈(약 3×10^9 뉴클레오타이드 ≈ 1GB)은 뇌의 모든 시냅스(~10^11 뉴런 × >10^3 시냅스 ≈ 3.5×10^15비트)를 명시할 수 없으므로 "배선 규칙(wiring rules)"만 부호화한다(게놈 병목). 해마 장소세포의 지도 형성 성향은 선천, 특정 지도는 학습; 얼굴 처리 회로는 선천, 개별 얼굴 인식은 학습. AI 함의: 귀납 편향·아키텍처(진화의 표적)·전이 가능한 일반 원리 — [bioRxiv 2019](https://www.biorxiv.org/content/10.1101/582643v1.full)
- "아이처럼 AI를 기르기"의 철학적 기초(2025): 핵심 지식은 발달형 AI에 선천적이어야 하고 복잡 기술은 그 위에서 학습되어야 하며, 인지 프로토타이핑으로 생성한 합성 데이터로 멀티모달 LM에 핵심 지식을 주입하자고 제안 — [arXiv 2502.10742](https://arxiv.org/abs/2502.10742)

### Inferences
- "핵심 지식"의 경험적 핵심(생후 수개월 내 대상·수·행위자 민감성)은 비교적 견고하지만, 그것이 '선천적 개념'인지 '빠르게 학습되는 지각 통계+초기 주의 편향'인지는 여전히 해석 문제다. Orhan & Lake/Vong 결과는 "강한 영역특정 선천 구조 없이도 상당 부분 학습 가능"을 보였을 뿐 "선천 구조가 없다"를 보인 것은 아니다(두 연구 모두 이를 명시).
- 설계 관점에서 양 진영의 교집합은 (i) 객체 단위 분절(objectness)·수량·행위자 탐지를 향한 *약한* 귀납 편향 또는 주의 편향, (ii) 그 위에서의 통계/연합 학습, (iii) Zador식으로 "가중치가 아니라 배선 규칙"을 하드코딩한다는 원칙이다.
- Karmiloff-Smith와 Smith & Thelen은 "스테이지"를 선천 모듈의 성숙이 아니라 신체·과제·기억의 상호작용에서 자기조직되는 것으로 보므로, 기계 유아에서도 단계는 하드코딩 스케줄이 아니라 역량·입력 통계·신체 제약의 변화에서 창발하도록 설계할 근거가 된다.

### Gaps
- Spelke 2022 원문(서론·결론) 및 BBS 정리(précis)는 PDF/429 오류로 직접 열람하지 못했고, 서평 초록과 2007 논문으로 주장 구조를 확인했다. 책에서 "다섯 번째 사회 시스템"의 지위가 2007보다 어떻게 갱신되었는지는 미확인.
- 응시시간 패러다임 자체의 재현성(예: ManyBabies 규모의 기대 위반 복제)에 대한 2020년대 체계적 평가 자료는 이번 조사에서 확보하지 못했다.

---

## 2. 출생 시 존재하는 본능적·피질하 시스템: 반사, 항상성 추동, 정서 시스템(Panksepp), 얼굴 선호, 모방, 수반성 탐지, 애착

### Takeaway
출생 시 확실히 존재하는 것은 (i) 원시 반사(근원·빨기·모로·파악 등; 2~6개월에 통합), (ii) 피질하 1차 정서 시스템(SEEKING, FEAR, RAGE, LUST, CARE, PANIC/GRIEF, PLAY; PAG·시상하부·편도·측좌핵에 분포, 신피질 없이도 작동), (iii) 얼굴형 자극에 대한 피질하 정향 편향(CONSPEC), (iv) 자기 행동과 감각 결과 사이의 수반성(contingency) 탐지 및 약 3개월의 "완벽→불완전 수반성 선호 전환", (v) 신생아 모방(메타분석 d≈0.68이지만 이질성이 크고 대규모 종단연구는 무효; 논쟁 중)이다. 애착은 출생 시 '행동 시스템'으로 준비되지만 특정 인물에 대한 명확한 애착(낯가림·분리불안)은 7개월 이후에 나타난다.

### Cited Findings
- 원시 반사: 근원(rooting) 반사는 재태 28주경 발달해 생후 약 4~6개월 지속; 모로(놀람) 반사는 재태 25주부터 보이며 출생 시 완전히 존재하고 6개월까지 통합; 파악(grasp) 반사는 약 6개월 지속하며 의도적 잡기의 기초 운동 패턴을 놓는다. 2~4개월에 근원·빨기 반사가 의도적 수유로 대체되고, 4~6개월에 파악·모로·비대칭 긴장성 경반사가 통합되어 의도적 뻗기·구르기가 가능해진다 — [Harkla(2차 자료)](https://harkla.co/blogs/special-needs/infant-reflexes); [HealthyChildren.org(AAP)](https://www.healthychildren.org/English/ages-stages/baby/Pages/newborn-reflexes.aspx); [URMC](https://www.urmc.rochester.edu/encyclopedia/content?ContentTypeID=90&ContentID=P02630)
- Panksepp의 7개 1차 정서 시스템: 긍정가 SEEKING(기대·탐색, 도파민), CARE, PLAY, LUST; 부정가 RAGE, FEAR, PANIC/SADNESS(분리 고통, 오피오이드 매개). 뇌간 상부·변연계 피질하 구조(PAG, 시상하부, 편도, 측좌핵)에 집중되며, 전기뇌자극(동물이 스스로 활성화/회피), 약리(오피오이드가 강아지 분리 고통 감소), 병변(PAG→시상하부→편도의 위계) 증거로 확인되었다. 1차(선천 피질하)·2차(학습 연합)·3차(신피질 인지) 과정의 3층 구조를 상정하며, 신생아기에 탈피질된 쥐도 1차 정서를 보여 신피질 없이 작동함을 시사한다 — [Davis & Montag 2019, Frontiers in Neuroscience](https://www.frontiersin.org/journals/neuroscience/articles/10.3389/fnins.2018.01025/full)
- 1차 정서 시스템의 개인차(ANPS)와 Big Five 성격의 메타분석적 연관이 보고되었다(2021) — [Scientific Reports 2021](https://www.nature.com/articles/s41598-021-84366-8)
- 얼굴 선호: Johnson & Morton(1991)의 CONSPEC 모델에 따르면 신생아의 얼굴 정향은 피질하 시스템이 담당하며(타원 경계, 상부 편중 특징 패턴, 양성 대비 같은 단순 기하 특성), 이것이 피질에 정보를 공급해 이후 얼굴 특이적 피질 시스템이 구축된다 — [Johnson 2005, Subcortical face processing (Review)](https://www.researchgate.net/publication/7492601_Subcortical_face_processing_Review); [Frontiers 2018 논의](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2018.00222/pdf)
- 신생아 모방 논쟁: Oostenbroek et al.(2016)의 최대 규모 종단연구는 4개 시점(1·3·6·9주)에서 9개 동작 중 어느 것도 신생아가 모방한다는 증거를 찾지 못했다(평균 Spearman r_s=0.027). Meltzoff et al.(2018)은 그 설계의 11개 결함을 지적하고 재분석에서 네 연령 모두 혀 내밀기 모방이 유의하다고 반박했다. Davis et al.(2021)의 메타분석(26편, 33개 독립 표본, 336개 효과크기)은 유의한 모방 효과(d=0.68, 95% CI [0.39, 0.96])를 보고했으나 연구 간 이질성이 컸고, Redshaw et al.(2020)은 신생아 "모방"의 개인차가 이후 사회인지를 예측하지 못한다고 보고했다 — [Meltzoff 2018, Dev Sci](https://onlinelibrary.wiley.com/doi/abs/10.1111/desc.12609); [Davis 2021, Perspectives on Psychological Science](https://journals.sagepub.com/doi/abs/10.1177/1745691620959834); [Redshaw 2020, Dev Sci](https://onlinelibrary.wiley.com/doi/10.1111/desc.12892)
- 수반성 탐지: Gergely & Watson은 영아가 선천적 "수반성 탐지 모듈"을 갖추어 신체적 자기의 1차 표상을 세우고 이후 반응적 사회 대상에 정향한다고 제안했다. 모듈의 목표값은 처음엔 완벽한 반응-수반 자극 탐색으로 설정되어 있다가 약 3개월경 "덜 완벽한" 사회적 수반성 선호로 전환된다("contingency switch"). 3개월 영아는 완벽/불완전 수반성을 변별하고, 모방 조건에서 더 오래 응시하며 정상 상호작용 조건에서 더 많이 미소 지었다. 저자들은 자폐의 1차 원인을 이 전환의 실패로 가설화했다 — [Gergely 2001, PubMed](https://pubmed.ncbi.nlm.nih.gov/11531136/); [Gergely & Watson, 사회적 바이오피드백 모델](https://www.semanticscholar.org/paper/Early-socio%E2%80%93emotional-development:-Contingency-and-Gergely-Watson/4b55eb43c4d2bd6ca62cbdf51b65926f7d5f1c4f); [Striano 2005, 1~3개월 사회적 수반성 민감성](https://www.eva.mpg.de/documents/Wiley-Blackwell/Striano_Sensitivity_DevScience_2005_1555283.pdf)
- 애착(Bowlby 단계): 전애착기(0~2개월, 무차별 반응), 애착 형성기(6주~7개월, 친숙한 사람 선호하되 누구의 돌봄도 수용), 명확한 애착기(7~24개월, 1차 양육자에 대한 특정 애착·매달리기·따라가기·분리불안·낯가림). 낯가림은 7~9개월경 친숙/비친숙 변별 능력의 발달과 함께 나타난다 — [SimplyPsychology(2차 자료)](https://www.simplypsychology.org/stages-of-attachment-identified-by-john-bowlby-and-schaffer-emerson-1964.html)
- 초기 사회적 주의 타임라인: 2개월 양자(dyadic) 주의 교환, 6개월 타인 시선 방향 추종·시선 확인, 9개월 삼자(triadic) 공동주의와 사회적 참조, 12개월 가리키기를 의도적인 것으로 이해, 18개월 시야 밖으로의 시선 추종 — [Wikipedia: Joint attention](https://en.wikipedia.org/wiki/Joint_attention)
- 초기 메타인지 지표: 12개월 영아는 지각 결정 후 오답 시 ERN(오류 관련 음전위)을 보이고, 정답 선택에 더 오래 지속(persistence)하는 등 결정 신뢰도를 평가한다; 20개월 영아는 숨긴 물건 위치에 대한 불확실성이 클 때 선택적으로 도움을 요청한다 — [Goupil & Kouider 2016, PubMed](https://pubmed.ncbi.nlm.nih.gov/27773566/); [Goupil, Romand-Monnier & Kouider 2016, PNAS](https://www.pnas.org/content/113/13/3492)
- 뇌 성장: 총 뇌 부피는 생후 1년에 101%, 2년차에 15% 증가하며 회백질은 1년차에 149% 증가; 2세에 성인 부피의 80~90% — [Knickmeyer et al. 2008, J Neurosci](https://www.jneurosci.org/content/28/47/12176). 시냅스 밀도는 1~2세에 성인의 약 150%로 정점(Huttenlocher 1979); 1차 시각피질은 4~12개월에 정점(성인의 140~150%), 전전두피질 시냅스 생성은 8개월에 정점기에 들어가 2년차까지 지속; 밀도는 청소년기에 급감 후 안정 — [Normal Development of Brain Circuits, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC3055433/)
- 영아 시각 입력의 자연 커리큘럼: 34명(1개월~2세)의 143시간 헤드캠 자료에서 생후 1년 동안 시야 내용이 "얼굴 위주"에서 "손과 손의 객체 조작 위주"로 체계적으로 전환된다(Fausey, Jayaraman & Smith 2016; Smith et al. 2018) — [Smith et al. 2018, TiCS](https://doi.org/10.1016/j.tics.2018.02.004); [From faces to hands, Cognition 2016](https://www.sciencedirect.com/science/article/am/pii/S0010027716300609)

### Inferences
- 출생 시 "지식"보다 확실한 것은 **가치·주의·행동 프리미티브**다: 정서/항상성 시스템(SEEKING = 탐색 추동의 기본 음조, PANIC/GRIEF = 애착의 벌점 신호, PLAY = 사회적 연습 추동), 얼굴·생물학적 운동 같은 자극으로의 정향 편향, 자기 행동의 결과를 추적하는 수반성 탐지기. 기계 유아에서 "선천"으로 넣을 1순위는 개념이 아니라 이런 **보상·주의 편향·자기-타자 분리 신호**다.
- 수반성 스위치(완벽→불완전)는 "자기 신체 발견 → 사회 파트너 발견"의 순서를 만드는 저비용 메커니즘으로, 단일 파라미터 변화로 발달 단계를 유도하는 설계 예시가 된다.
- 반사는 "학습 전 부트스트랩 행동"(빨기·잡기)이며 의도적 행동이 등장하면 억제·통합된다 — 기계 유아의 초기 행동 정책에 고정 루틴을 넣되 학습된 정책이 이를 덮어쓰도록 하는 구조와 대응한다.

### Gaps
- 항상성 추동(배고픔·체온·통증 회피)의 신생아 행동 조직에 관한 1차 문헌은 이번 조사에서 직접 확보하지 못했다(Panksepp 자료의 SEEKING/항상성 언급으로 간접 확인).
- 반사·애착 단계 자료는 교육용 2차 자료(AAP, SimplyPsychology 등)에 의존했다. 연령 수치는 널리 통용되나 원전 확인은 못 했다.

---

## 3. Piaget, Vygotsky, Tomasello, Gopnik, Karmiloff-Smith(및 Smith & Thelen): 각각이 기제적 설명에 기여하는 것

### Takeaway
Piaget는 "동화·조절·평형화"라는 자기수정 학습 루프와 감각운동 → 상징 → 조작의 순서를, Vygotsky는 "혼자 가능한 것과 도움받아 가능한 것 사이(ZPD)"에서 작동하는 수반적 비계(scaffolding)와 언어의 도구화를, Tomasello는 9개월의 공동 지향성(joint intentionality)과 3세의 집단 지향성(collective intentionality)이라는 사회인지적 전환을, Gopnik은 긴 아동기를 "탐색 후 활용"이라는 explore-exploit 해법으로, Karmiloff-Smith는 암묵 지식이 점진적으로 명시화·모듈화되는 표상 재기술을 제공한다. 공통 교훈은 발달이 "단계 라벨"이 아니라 학습자-신체-환경-타인 상호작용의 역학에서 나온다는 것이다.

### Cited Findings
- Piaget: 감각운동기(0~2세), 전조작기(2~7세), 구체적 조작기(7~11세), 형식적 조작기(11세+). 감각운동기 6하위단계: 단순 반사(0~6주), 1차 순환반응(6주~4개월), 2차 순환반응(4~8개월), 2차 반응의 협응(8~12개월), 3차 순환반응·새로움·호기심(12~18개월), 도식의 내면화(18~24개월). 기제는 동화(assimilation)·조절(accommodation)·평형화(equilibration)·도식(schema). 비판: 영아 능력 과소평가(약 3개월에 대상 물리 이해의 "핵심 지식" 증거), 단계 불연속성 부정(영역별 숙달 시기가 다름), 사회문화 요인 과소평가 — [Wikipedia: Piaget's theory](https://en.wikipedia.org/wiki/Piaget%27s_theory_of_cognitive_development)
- Vygotsky/ZPD: ZPD는 "학습자가 혼자 할 수 있는 것과 도움을 받아도 할 수 없는 것 사이의 공간"이며, 독립 수행이 아닌 성인 보조 하의 문제해결 능력을 평가해야 한다고 보았다. Wood, Bruner & Ross(1976)가 ZPD를 적용해 "비계(scaffolding)" 개념을 만들었고, 비계는 학습자 역량이 늘면 점진적으로 제거되는 임시 지원이다; "비계로 가능한 가장 어려운 과제를 주는 것이 가장 큰 학습 이득"을 낳는다 — [Wikipedia: ZPD](https://en.wikipedia.org/wiki/Zone_of_proximal_development)
- 비계의 수반성: "수반적 비계(contingent scaffolding)"는 지원의 종류와 양을 그 순간 학습자의 필요에 맞추는 것이며, 종단연구에서 어머니의 3~4세 아동에 대한 언어적 비계(특히 놀이 중 명시적 개념 연결 제공)가 6세의 작업기억·언어 기술과 상관했다; 지도 수준이 높을수록 효과가 크지만 과도하거나 부적합한 지도는 수행을 저해한다 — [Wikipedia: Instructional scaffolding](https://en.wikipedia.org/wiki/Instructional_scaffolding)
- Tomasello(*Becoming Human*, 2019): 인간 고유성은 "공유 지향성(shared intentionality)"에서 나온다. 약 9개월에 "공동 지향성(joint intentionality)"이 출현해 양육자와 대상을 함께 지각함을 재귀적으로 인식하고("나는 네가 내가 안다는 것을 안다"), 약 3세에 "집단 지향성(collective intentionality)"이 출현해 규칙·규범을 공동체의 기준으로 이해하며 성인을 문화적 지식의 대표로 본다. 사회인지·의사소통·문화학습·협력적 사고·협동·친사회성·사회규범·도덕적 정체성의 8개 경로가 인간을 대형 유인원과 구분하며, 유인원의 협력은 "순수하게 도구적"이다 — [Goodreads 책 소개](https://www.goodreads.com/en/book/show/39217056-becoming-human); [Amazon](https://www.amazon.com/Becoming-Human-Ontogeny-Michael-Tomasello/dp/0674980859)
- Tomasello 비판: Moll, Nichols & Mackey(2020)는 "인간 발달과 공유 지향성 가설의 재고"에서 이 가설을 재검토했다(세부 내용 미확인) — [USC PDF](https://dornsife.usc.edu/henrike-moll/wp-content/uploads/sites/404/2023/11/Moll_Nichols_Mackey_2020.pdf)
- Tomasello/Bruner 개념의 AI 구현: "SocialAI School"(2023)은 Tomasello와 Bruner 이론의 사회인지 능력(문화 진입을 가능케 하는 능력)을 절차적 생성 환경 스위트로 운영화하고 RL 에이전트와 LLM으로 실험했다 — [arXiv 2307.07871](https://arxiv.org/abs/2307.07871)
- Gopnik(2020, Phil Trans B): "길고 보호된 인간 아동기라는 생활사 진화가 목표 지향적 활용(exploitation)의 요구가 시작되기 전, 폭넓은 가설 탐색과 탐험의 초기 시기를 허용한다." 이 인지 프로파일은 다른 동물에도 있으며 신기성 선호(neophilia)·놀이와 연관된다. 저자는 이를 explore-exploit 트레이드오프·탐색·샘플링의 계산적 아이디어 및 신경과학 소견과 연결하고, 어린 학습자가 외부 정보 탐색과 가설 공간 탐색 모두에서 매우 탐험적이며 "때로는 더 나이 든 학습자와 성인보다 더 탐험적"이라는 여러 경험적 증거를 제시한다 — [Gopnik 2020 초록(Semantic Scholar)](https://api.semanticscholar.org/graph/v1/paper/DOI:10.1098/rstb.2019.0502?fields=title,abstract,year,citationCount); [Phil Trans B](https://royalsocietypublishing.org/doi/10.1098/rstb.2019.0502)
- Gopnik 관련 경험 연구(제목만 확인): Lucas, Bridgers, Griffiths & Gopnik(2014, Cognition) "아동이 성인보다 더 나은(최소한 더 열린) 학습자일 때: 인과관계 형태 학습의 발달 차이"; Gopnik et al.(2017, PNAS) "아동기→청소년기→성인기 생활사에 걸친 인지 유연성과 가설 탐색의 변화" — [Cognition 2014 DOI](https://doi.org/10.1016/j.cognition.2013.12.010); [PNAS 2017 DOI](https://doi.org/10.1073/pnas.1700811114)
- Karmiloff-Smith: 표상 재기술(RR)은 "인지 시스템 안에 있는 정보가 그 시스템에게 점진적으로 명시적 지식이 되는 과정"이며, 모듈화는 발달의 출발점이 아니라 결과다. 인지적 유연성은 암묵적 지각 지식이 명시적·메타표상적 형태로 재구조화되면서 생긴다 — [PhilPapers: Précis](https://philpapers.org/rec/KARPOB); [APS 추모 기사](https://www.psychologicalscience.org/observer/remembering-annette-karmiloff-smith)
- Smith & Thelen(1999 Psych Review; 2003 TiCS): A-not-B 오류는 일반적 목표 지향 도달 과정의 역학(기억 흔적·현저성·지연·자세의 결합)에서 나오며, 수학적 모델과 로봇 구현까지 제시해 "체화되고 역동적인" 발달을 주장했다 — [Smith & Thelen 2003 PDF](http://wexler.free.fr/library/files/thelen%20(2003)%20development%20as%20a%20dynamic%20system.pdf); [Wikipedia: A-not-B error](https://en.wikipedia.org/wiki/A-not-B_error); [Thelen 기여 회고, 2006](https://link.springer.com/article/10.1162/biot.2006.1.1.87)

### Inferences
- 기제 수준으로 번역하면: Piaget = 예측오차 기반 모델 수정 루프(동화/조절) + 행동→표상의 내면화; Vygotsky = 교사가 학습자 역량 추정치에 수반해 난이도를 조절하는 외부 커리큘럼 컨트롤러; Tomasello = 삼자 공동주의라는 "공유 참조 프레임"의 등장이 언어·문화 학습의 전제조건; Gopnik = 탐색 온도(또는 사전 분포의 넓이)를 아동기에 높게 두고 점진적으로 낮추는 스케줄; Karmiloff-Smith = 절차적 성공 후 표상을 재기술해 접근 가능하게 만드는 오프라인 재구성(압축·추상화) 단계.
- Smith & Thelen이 보여주듯 동일 "지식"이 자세·지연 같은 비인지 변수에 의해 나타나거나 사라지므로, 기계 유아 평가도 단일 과제 통과/실패 대신 다중 조건에서의 수행 지형으로 해야 한다.

### Gaps
- Gopnik의 구체적 과제·연령 데이터(어떤 과제에서 아동이 성인을 능가했는지)는 2014·2017 논문 초록이 API에서 제공되지 않아 제목 수준으로만 확인했다.
- Tomasello *Becoming Human*은 출판사·서평 요약에 의존했으며, Moll et al. 2020 비판의 구체 논점은 미확인.
- Vygotsky의 "사적 언어(private speech)"가 자기조절을 매개한다는 실증 자료는 확보하지 못했다.

---

## 4. 내재적 동기와 호기심(Oudeyer & Kaplan, Gottlieb, Kidd) 및 양육자의 학습 형성(motherese, 공동주의, 수반적 반응성)

### Takeaway
영아는 외적 보상 없이 "학습 진전(learning progress)"이 있는 활동과 "너무 단순하지도 너무 복잡하지도 않은(Goldilocks)" 자극에 주의를 배분하며, 기대 위반은 탐색과 학습을 선택적으로 증폭시킨다(Stahl & Feigenson 2015). Oudeyer & Kaplan의 계산 모델은 예측오차 극대화가 아닌 **오차 감소율(학습 진전)** 을 보상으로 삼아야 "흰 벽/잡음" 문제를 피하고 발달 궤적이 자기조직됨을 보였다. 양육자는 motherese(IDS 선호는 67개 실험실 복제에서 d=0.35로 확인), 공동주의 매칭 발화, 생방송 상호작용(Kuhl의 social gating)과 수반적 비계로 입력의 통계와 타이밍을 아동의 상태에 맞춰 준다.

### Cited Findings
- Oudeyer & Kaplan(2007) 유형학: 지식 기반 모델(정보이론적: 불확실성·정보이득·분포적 놀람; 예측적: 새로움(novelty, 높은 예측오차)·학습 진전(LPM, 시간에 따른 오차 감소)), 역량 기반 모델(무능 극대화·역량 진전/몰입·역량 극대화), 형태학적 모델(동기성·안정성·분산). 순진한 예측오차 접근의 문제: "매우 다른 감각운동 상황 간 비교"가 무의미한 보상을 낳는다(예측 불가능한 나뭇잎 흔들림에서 정적인 흰 벽으로 옮겨가는 것을 보상). LPM은 감각운동 공간을 영역으로 분할해 일관된 맥락 안에서의 오차 감소를 추적해 이를 해결한다. 심리학 기원: White(1959) 효능 동기, Berlyne(1960) 최적 불일치, Csikszentmihalyi(1991) 몰입. Playground 실험에서 LPM 로봇은 무작위 옹알이 → 신체 놀이 → 대상 지향 행동 → 행동유도성(affordance) 발견으로 이어지는 발달 궤적을 자기조직했다 — [Frontiers in Neurorobotics 2007](https://www.frontiersin.org/articles/10.3389/neuro.12.006.2007/full)
- Gottlieb & Oudeyer(2018, Nature Reviews Neuroscience) "능동 샘플링과 호기심의 신경과학을 향하여"가 존재한다(초록 미확인) — [NRN 2018 DOI](https://doi.org/10.1038/s41583-018-0078-0); [Neurocuriosity 프로젝트](https://flowers.inria.fr/neurocuriosityproject/)
- Kidd, Piantadosi & Aslin(2012) Goldilocks 효과: 7~8개월 영아 72명(두 실험)이 상자 뒤에서 물건이 나타나는 애니메이션을 보는 동안, 사건의 확률(0~1)로 복잡도를 정량화했을 때 사건이 너무 예측 가능해도, 너무 놀라워도 시선을 돌렸다(중간 복잡도에서 주의 최대) — [ScienceDaily 2012](https://www.sciencedaily.com/releases/2012/05/120523200252.htm); 청각 영역 복제 — [Kidd et al. 2014, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC4807134/)
- Stahl & Feigenson(2015, Science): 11개월 영아 110명에게 대상 행동에 대한 기대를 위반하거나 확인하는 사건을 보여준 결과, 기대 위반 대상에 대해 정보를 더 효율적으로 학습하고, 탐색을 늘렸으며, 본 위반의 종류에 특이적인 가설 검증 행동(예: 고체성 위반 후 두드리기)을 보였다. 저자들은 "초기 생애에 기대 위반은 무엇을 학습할지의 문제에 쐐기를 제공한다"고 결론 — [Science 2015 DOI](https://doi.org/10.1126/science.aaa3799)
- 학습 진전의 최신 검증(de Boer, Poli, Meyer & Hunnius, Developmental Science 2026): 3.5세 아동 56명의 자유 놀이에서 수행 기반 LP와 정보 기반 LP는 약한 상관만 보였지만 둘 다 탐색에 영향을 주었고, 수행 기반 LP는 모든 장난감에서 참여를 예측한 반면 정보 기반 LP는 한 장난감에서만 예측해 "맥락의 역할"을 부각했다 — [Dev Sci 2026](https://onlinelibrary.wiley.com/doi/10.1111/desc.70229)
- 학습 진전 가설 요약: 아동은 자신의 학습 진전을 모니터해 행동을 조절한다 — 진전이 높으면 내재적 보상으로 탐색을 지속하고, 낮으면 이탈해 주의를 옮긴다 — [Dev Sci 2026](https://onlinelibrary.wiley.com/doi/10.1111/desc.70229); [Oudeyer 2016, Topics in Cognitive Science](https://onlinelibrary.wiley.com/doi/10.1111/tops.12196)
- Gopnik(2020): 아동기 = 탐색 우선 시기(위 3절 참조) — [Phil Trans B 2020](https://royalsocietypublishing.org/doi/10.1098/rstb.2019.0502)
- Motherese(IDS) 선호 복제: ManyBabies 1(2020)은 북미·유럽·호주·아시아 67개 실험실에서 세 방법(head-turn preference, central fixation, eye tracking)으로 IDS 대 ADS 선호를 측정해 메타분석 효과크기 d=0.35(95% CI [0.29, 0.42])를 얻었다. 이는 0보다 유의하지만 선행 문헌 메타평균(0.67)보다 작았고, 2,329명 중 1,373명(58.95%)이 IDS를 수치상 선호했다. 선호는 월령이 높을수록, 자극이 모국어·방언과 일치할수록, HPP 절차에서 더 강했다 — [ManyBabies Consortium 2020, AMPPS](https://journals.sagepub.com/doi/10.1177/2515245919900809); [MB1 메타분석 일관성(Open Mind)](https://direct.mit.edu/opmi/article/doi/10.1162/opmi_a_00134/120611/Evidence-for-Infant-directed-Speech-Preference-Is)
- 사회적 게이팅(Kuhl, Tsao & Liu 2003): 9~10개월 영어권 영아에게 12회 만다린어 세션을 제공했을 때, 만다린 화자와 **대면 상호작용**한 영아만 10~11개월에 만다린 특이 음소 변별 능력이 회복되었고, 동일 내용을 오디오-비주얼 비디오나 오디오 단독으로 받은 영아는 감퇴가 역전되지 않았다 — [Friendly et al. 2013, Frontiers in Psychology(Kuhl 2003 요약)](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2013.00718/full)
- 공동주의와 언어 입력 정렬: 전형 발달 아동에서는 어머니 발화의 최대 78%가 아동이 주목 중인 대상과 일치하는 반면, 언어 지연 아동에서는 50%에 그친다; 공동주의 에피소드는 발화 이해를 가능케 하는 환경 맥락을 제공한다 — [Wikipedia: Joint attention](https://en.wikipedia.org/wiki/Joint_attention)
- 수반적 반응성: Gergely & Watson의 사회적 바이오피드백 모델에서 양육자의 "표시된(marked)" 정서 반영(불완전 수반성)이 영아의 자기 정서 상태 표상을 형성한다(2절 참조) — [Semantic Scholar](https://www.semanticscholar.org/paper/Early-socio%E2%80%93emotional-development:-Contingency-and-Gergely-Watson/4b55eb43c4d2bd6ca62cbdf51b65926f7d5f1c4f); 이전 상호작용의 수반성이 이후 영아 행동에 미치는 효과(Bigelow & Birch 1999) — [PDF](https://kidlab-psych.sites.olt.ubc.ca/files/2019/06/BigelowBirchInfantBehaviorDev1999.pdf)
- Frank(2023, TiCS): 아동이 LLM보다 훨씬 적은 데이터로 학습하는 이유의 후보로 기존 개념 지식, 멀티모달 접지(grounding), 입력의 상호작용적·사회적 성격을 든다 — [TiCS 2023](https://www.sciencedirect.com/science/article/abs/pii/S1364661323002036)

### Inferences
- 설계 규칙: 호기심 보상은 "놀람" 자체가 아니라 **맥락별(영역별) 예측오차의 감소율**로, Goldilocks 효과는 이 보상이 자연히 만들어내는 주의 배분 패턴으로 해석할 수 있다. 기대 위반은 "해당 객체에 대한 학습률/탐색 예산 증가" 트리거로 구현할 수 있다(Stahl & Feigenson의 위반 특이적 탐색은 가설 검증 행동 생성기를 시사).
- 양육자 효과의 기제적 핵심은 (1) 입력의 **시간적 정렬**(아동이 보는 것에 대해 말하기), (2) **수반성**(아동 행동 직후의 반응), (3) 난이도의 **수반적 조절**(비계)이다. Kuhl의 결과는 동일 콘텐츠라도 상호작용성이 없으면 학습이 안 되는 시기가 있음을 시사하므로, 기계 유아의 "교사"는 녹화 재생이 아닌 반응형 에이전트여야 한다.
- IDS 선호 효과크기가 문헌 추정보다 작다는 사실(d=0.35)은 motherese를 "필수 입력 포맷"이 아니라 "주의 포획 보조"로 가중치를 낮춰 설계할 근거다.

### Gaps
- Kuhl 2003 원문 초록(노출 총 시간, 표본 수)은 PNAS 접근 차단으로 2013년 리뷰의 기술에 의존했다.
- Gottlieb & Oudeyer 2018의 구체 신경 기제(예: 정보 추구의 도파민/안와전두 관여)는 초록이 제공되지 않아 기술하지 못했다.
- 수반적 반응성이 학습 효율을 높인다는 **정량** 추정치(예: 효과크기)는 확보하지 못했다.

---

## 5. 결정적/민감기, 발달 타이밍, 발달이 단계적인 이유(Elman "starting small", Newport "less is more", 지각적 협소화, 시냅스 가소성 브레이크)

### Takeaway
민감기는 (i) 입력 통계에 맞춘 지각 체계의 "협소화"(음소 6~8→10~12개월, 얼굴 3~9개월)로 행동 수준에서, (ii) PV+ 억제 뉴런 성숙과 분자적 "브레이크"(perineuronal net 등)의 상향 조절로 회로 수준에서 관찰된다. "작게 시작하기"(제한된 기억·단순 입력이 학습을 돕는다)는 Elman(1993)·Newport(1990)·Kersten & Earles(2001)의 지지와 Rohde & Plaut(1999, 2003)의 반증이 공존하는 논쟁 영역이지만, 영아 헤드캠 비디오로 학습한 최근 모델(Sheybani 2023)은 "가장 어린 시기의 느리고 단순한 입력부터" 학습하는 커리큘럼이 가장 좋은 표상을 준다고 보고했다.

### Cited Findings
- 음소 협소화: 6~8개월 영어권 단일언어 영아는 힌디어 성인만큼 비모국어 대비를 변별했으나 10~12개월에는 실패했다(Werker & Tees 1984); 6~12개월 사이 모국어 변별은 향상되고 비모국어 변별은 급감한다 — [Friendly et al. 2013, Frontiers](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2013.00718/full); [Wikipedia: Perceptual narrowing](https://en.wikipedia.org/wiki/Perceptual_narrowing)
- 얼굴 협소화: 6개월 영아는 원숭이 얼굴 둘을 사람 얼굴만큼 잘 변별하지만 9개월과 성인은 사람 얼굴에 훨씬 능하다(Pascalis 2002); 타인종 얼굴 인식은 3개월(모든 인종) → 9개월(자기 인종만)로 좁아진다; 7개월에는 목소리에서 "자기 언어 효과"가 나타난다. Scott & Monesson(2009): 원숭이 얼굴에 **개체 수준 이름 라벨링**을 받은 영아만 9개월에 변별을 유지했다(범주 수준 노출은 불충분) — [Friendly et al. 2013](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2013.00718/full); [Wikipedia: Perceptual narrowing](https://en.wikipedia.org/wiki/Perceptual_narrowing)
- 조산아의 비정형 지각 협소화는 2세 언어 습득 저하와 연관된다 — [BMC Neuroscience 2010](https://bmcneurosci.biomedcentral.com/articles/10.1186/1471-2202-11-88); 감각 간(intersensory) 말소리 지각의 협소화 — [PNAS 2009](https://www.pnas.org/doi/10.1073/pnas.0904134106)
- 회로 기제(Hensch): 결정적 시기의 개시·종료를 결정하는 "트리거"와 "브레이크"가 있으며, 파브알부민(PV)+ 억제 뉴런이 결정적 시기 창과 함께 성숙한다. 가소성은 나이와 함께 단순히 사라지는 것이 아니라 분자적 브레이크의 발달적 상향 조절로 능동 억제되며, 브레이크를 해제하면 성체 시각피질의 가소성이 회복된다. 결정적 시기 종료는 PV+ 뉴런을 감싸는 perineuronal net(PNN) 응축으로 표시된다 — [Hensch, Factors that initiate and terminate critical periods (PDF)](https://www.esforum.de/publications/PDFs/sfr25/SFR25_05_Hensch.pdf); [Hensch 2005, Nat Rev Neurosci](https://www.nature.com/articles/nrn1787); [PNN 리뷰, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC12614057/); [Hensch 2022, J Vision "Lifting brakes"](https://henschlab.mcb.harvard.edu/papers/reviews-commentaries)
- 시냅스 과잉생성→가지치기 타임라인(2절 참조): 시각피질 4~12개월 정점, 전전두 8개월부터 2년차까지 정점기, 청소년기 급감 — [PMC: Normal Development of Brain Circuits](https://pmc.ncbi.nlm.nih.gov/articles/PMC3055433/)
- "Starting small"(Elman 1993)과 "Less is more"(Newport 1990): 아동의 제한된 정보처리 능력이 더 단순한 입력을 끌어내거나(외적 소재) 제한된 인지 자원 자체가 학습을 돕는다(내적 소재)는 가설. Kersten & Earles(2001)는 성인이 소형 인공언어를 처음에 작은 조각(개별 단어)만 제시받고 나중에 복잡 문장을 받을 때 의미·형태론을 더 잘 학습함을 보였다 — [Kersten & Earles 2001, JML](https://www.sciencedirect.com/science/article/abs/pii/S0749596X00927517); [Elman 1993, Cognition DOI](https://doi.org/10.1016/0010-0277(93)90058-4)
- 반증: Rohde & Plaut(1999)는 순환망의 언어 학습이 starting small에 의존하지 않으며, 점진적 의미 제약을 도입해 언어를 더 현실적으로 만들수록 그런 제한이 오히려 습득을 저해함을 보였다("less is less", 2003) — [Rohde & Plaut 1999, PubMed](https://pubmed.ncbi.nlm.nih.gov/10520565/); [Rohde & Plaut 2003 PDF](https://ni.cmu.edu/~plaut/papers/pdf/RohdePlaut03chap.less-is-less.pdf); 중심 내포 구조 학습에서의 starting small 조건 연구 — [PMC6585836](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6585836/)
- 영아 데이터 기반 커리큘럼(Sheybani et al., NeurIPS 2023 Spotlight): 헤드캠 영아 비디오를 월령별로 나누어 자기지도 모델을 학습시켰을 때 **가장 어린 집단의 데이터로 시작**하는 것이 가장 강한 학습 신호와 최고의 하류 성능을 냈으며, 그 이점은 초기 시각 경험의 **느림과 단순함** 때문이었다 — [NeurIPS 2023 논문 PDF](https://proceedings.neurips.cc/paper_files/paper/2023/file/a9ad92a81748a31ef6f2ef68d775da46-Paper-Conference.pdf); [코드](https://github.com/ssheybani/baby-vision-curriculum)
- 발달하는 영아가 만드는 커리큘럼(Smith et al. 2018): 감각운동 기술이 발달하면서 통계 학습에 쓸 수 있는 훈련 집합이 점진적으로 정제되며, 이 변화하는 환경은 "여러 영역에 걸쳐 학습을 최적화하는 발달적으로 정렬된 커리큘럼"일 수 있다 — [TiCS 2018 DOI](https://doi.org/10.1016/j.tics.2018.02.004)
- 언어 LM에서의 커리큘럼 시도: BabyLM 2025에는 "Beyond Repetition: Text Simplification and Curriculum Learning", "Influence-driven Curriculum Learning", "Batch-wise Convergent Pre-training: Step-by-Step Learning Inspired by Child Language Development" 등이 제출되었다 — [ACL Anthology BabyLM 2025](https://aclanthology.org/events/babylm-2025/); 2024에는 변이 집합(variation sets) 주입이 효율을 높였다 — [arXiv 2411.09587](https://arxiv.org/abs/2411.09587)
- 아동의 입력 통계 위반·언어 창조·결정적 시기를 "데이터 격차 너머"의 증거로 드는 BBS 논평(2024)이 있다 — [Cambridge BBS](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/beyond-the-data-gap-children-create-languages-violate-their-input-statistics-and-exhibit-critical-periods/2BF9E7E520D6A309C3C18E8835315AD9)

### Inferences
- 단계적 발달의 두 독립 원천: (A) **입력 측** — 신체·운동 능력(목 가누기→손 뻗기→기기→걷기)이 입력 분포를 바꾸고 초기 입력이 느리고 단순하다(Smith; Sheybani); (B) **학습자 측** — 가소성 창이 영역별로 순차적으로 열리고 닫힌다(시각→전전두; Hensch 브레이크). 기계 유아는 (A)를 센서/행동 해상도의 점진적 해제로, (B)를 모듈별 학습률·정규화 스케줄(및 "브레이크" 해제 가능성)로 구현할 수 있다.
- 협소화는 손실이 아니라 **환경 통계에의 특화(전문화)** 이며, 개체 수준 라벨링(Scott & Monesson)이 협소화를 막는다는 결과는 "변별이 과제에 유용하면 유지된다"는 과제 의존적 가지치기 규칙을 시사한다.
- "starting small"의 논쟁은 과제 구조에 따라 결론이 달라짐을 보여주므로, 기계 유아에서 커리큘럼은 고정 처방이 아니라 학습 진전 신호로 자동 조절되는 것이 안전하다.

### Gaps
- Newport 1990 원문과 Elman 1993 초록은 직접 확인하지 못했다(2차 기술에 의존).
- 인간에서 PV/PNN 브레이크의 **시기 수치**(예: 시각 결정적 시기 종료 연령)는 동물 연구 중심 리뷰로만 확인했다.

---

## 6. 타임라인: 대상영속성, 자기인식(거울), 마음이론(틀린 믿음), 언어, 실행기능, 메타인지는 언제 나타나는가

### Takeaway
응시 기반 지표는 행동 기반 지표보다 수개월~수년 이르며, 이 격차 자체가 "표상의 존재 vs 표상을 행동에 쓰는 능력"의 분리를 보여준다. 확립된 순서: 수반성 전환(≈3개월) → 기대 위반 기반 대상 지식(3.5개월) → 지각 협소화·삼자 공동주의·A-not-B(6~12개월) → 첫 단어·가리키기 이해(12개월) → 50단어·어휘 폭발·두 단어 조합·거울 자기인식·명시적 불확실성 보고(18~24개월) → 집단 지향성(3세) → 명시적 틀린 믿음·DCCS 전환(4~5세) → 실행기능의 성인 수준 도달(청소년기). 암묵적 틀린 믿음(15개월)은 재현 실패가 누적되어 논쟁 중이며, 거울 자기인식 시기는 문화에 따라 크게 다르다.

### Cited Findings
- 대상영속성: 기대 위반에서 3.5개월(Baillargeon); Piaget의 완전한 대상영속성은 18~24개월; A-not-B 오류는 8~12개월에 나타나며 12개월 이상은 보통 범하지 않는다. 비판: 더 이른 출현, 덮개·위치에 따른 난이도 차, 운동 행위 의존성, A-not-B를 기억/공간 정위 한계로 보는 해석 — [Wikipedia: Object permanence](https://en.wikipedia.org/wiki/Object_permanence); [Wikipedia: A-not-B error](https://en.wikipedia.org/wiki/A-not-B_error)
- 사회적 주의: 2개월 양자 상호작용, 6개월 시선 추종, 8개월 원형 선언적 가리키기(여아에서 두드러짐), 9개월 삼자 공동주의·사회적 참조, 12개월 가리키기의 의도성 이해, 15개월 타인이 마음을 가짐을 인식, 18개월 시야 밖 시선 추종, 24개월 과거 사건으로의 주의 확장 — [Wikipedia: Joint attention](https://en.wikipedia.org/wiki/Joint_attention)
- Tomasello: 9개월 공동 지향성, 3세 집단 지향성 — [Goodreads](https://www.goodreads.com/en/book/show/39217056-becoming-human)
- 애착: 명확한 애착·낯가림·분리불안 7~24개월 — [SimplyPsychology](https://www.simplypsychology.org/stages-of-attachment-identified-by-john-bowlby-and-schaffer-emerson-1964.html)
- 언어 산출: 0~2개월 쿠잉, 4~6개월 옹알이 시작, 8~12개월 반복적(canonical, "dada")·변이적 옹알이, 10~12개월 소리 모방, 12~18개월 첫 단어(이해가 산출에 앞섬), 18개월 약 50단어, 18~24개월 어휘 폭발(fast mapping)과 두 단어 발화, 24~30개월 3~4단어 문장, 28~36개월 3단어 단순문 표준, 36~60개월 음운 인식·복문; 개인차가 크다 — [Wikipedia: Language development](https://en.wikipedia.org/wiki/Language_development)
- 언어 이해: "약 6~9개월부터 아동은 말소리 단어를 시각 대응물에 연결하며 첫 단어를 습득하기 시작한다" — [Vong et al. 2024, Science](https://doi.org/10.1126/science.adi1374)
- 거울 자기인식(MSR): 서구 아동은 18~24개월에 처음 나타난다(자기개념 출현의 지표). 문화 간 차이가 크다: 18~20개월에서 독일·그리스·코스타리카 도시 아동은 50% 이상이 통과했으나 카메룬 농촌 아동은 4% 미만이었다(Keller 등). Kärtner et al.(2012)은 276명(첫 평가 16~21개월)을 6주간 매주 평가한 종단 모노그래프에서 자율성을 중시·지원하는 사회문화 맥락에서 MSR이 더 일찍 출현한다는 가설을 검증했다; Broesch et al.(2010)은 7개 문화에서 유의한 변이를 보고했다 — [Explaining cross-cultural variation in MSR, PubMed 2021](https://pubmed.ncbi.nlm.nih.gov/34166010/); [Kärtner 2012 모노그래프 PDF](https://www.uni-muenster.de/imperia/md/content/psyifp/aekaertner/lit2012/k__rtner_et_al__2012__monograph_-_the_development_of_msr.pdf); [Broesch 2010 PDF](https://www2.psych.ubc.ca/~henrich/pdfs/Journal%20of%20Cross-Cultural%20Psychology-2010-Broesch-%20Cultural%20Variations%20in%20Children's%20Mirror%20Self-Recognition.pdf); [ni-Vanuatu 유아 MSR, 2026](https://www.frontiersin.org/journals/developmental-psychology/articles/10.3389/fdpys.2026.1690962/pdf)
- 메타인지: 언어적 메타인지 판단은 아동 후기까지 부정확하지만, 비언어 패러다임에서 결정 신뢰도 추정·오류 모니터링 같은 기본 형태는 언어 이전 영아에게도 존재한다(12개월 ERN, 20개월 도움 요청) — [Goupil & Kouider 2019, Current Directions DOI](https://doi.org/10.1177/0963721419848672); [PNAS 2016](https://www.pnas.org/content/113/13/3492); [12·24개월 불확실성 하 탐색, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC7118196/)
- 명시적 틀린 믿음: Wellman, Cross & Watson(2001)의 메타분석에서 아동은 4세경 일관되게 통과하며, 연령·국가·4개 과제 요인을 포함한 모델이 수행 분산의 55%를 설명했다 — [ERIC: Wellman et al. 2001](https://eric.ed.gov/?id=EJ639719); 문화 간 메타표상적 틀린 믿음 이해의 발달적 통일성과 변이(2026) — [PMC13338850](https://pmc.ncbi.nlm.nih.gov/articles/PMC13338850/)
- 암묵적 틀린 믿음 논쟁: 예기 응시(anticipatory looking) 과제의 부분/완전 재현 실패가 급증해 "재현 위기"로 불린다. Kulke et al.(2018)의 사전등록 대규모 복제에서 4개 영향력 있는 패러다임 중 1개만 재현되었고, 그마저 교란변수를 제거하면 재현되지 않았다. Kaltefleiter et al.(2022)의 대규모 종단 복제는 2~4세에서 목표 기반 예측은 지지했으나 틀린 믿음 기반 예측은 혼재된 증거를 얻었고, 3~4세의 믿음 일치 응시도 재현되지 않았다. 영아 자발 반응 과제의 메타분석과, 실패한 복제의 해석에 관한 Baillargeon·Buttelmann·Southgate(2018)의 반론 논평이 있다 — [Kulke et al. 2018, Psychological Science](https://doi.org/10.1177/0956797617747090); [Kaltefleiter 2022, Dev Sci](https://onlinelibrary.wiley.com/doi/10.1111/desc.13224); [Poulin-Dubois 등 생애 전반 비재현](https://www.sciencedirect.com/science/article/abs/pii/S0885201417300321); [자발 반응 과제 메타분석](https://www.sciencedirect.com/science/article/pii/S0163638319300414); [Baillargeon et al. 2018 논평 PDF](https://infantcognition.web.illinois.edu/wp-content/uploads/2021/12/Baillargeon_Buttelmann_Southgate_-2018-Invited-commentary-Interpreting-failed-replications-of-early-false-belief-findings.pdf)
- 실행기능: 3~5세에 큰 발달이 일어나며(Diamond 2013), DCCS(차원 전환 카드 분류)는 보통 4.5~5세까지 성공하지 못하지만 색을 도형이 아닌 배경 속성으로 만들면 3~3.5세에 성공한다; 실행기능은 아동기 내내 향상되어 청소년기에 성인 수준에 도달한다 — [Diamond 2013, Annual Review of Psychology PDF](https://www.devcogneuro.com/Publications/ExecutiveFunctions2013.pdf); [청소년기→성인기 EF 성숙의 정준 궤적, PMC](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10616171/); [영아기→아동 후기 EF 종단 발달, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC12311735/)
- Piaget 단계 연령(3절 참조): 감각운동 0~2세, 전조작 2~7세, 구체적 조작 7~11세, 형식적 조작 11세+ — [Wikipedia: Piaget's theory](https://en.wikipedia.org/wiki/Piaget%27s_theory_of_cognitive_development)
- 뇌: 2세에 성인 부피의 80~90%; 시냅스 밀도 1~2세 정점 — [J Neurosci 2008](https://www.jneurosci.org/content/28/47/12176); [PMC3055433](https://pmc.ncbi.nlm.nih.gov/articles/PMC3055433/)

### Inferences
- 이정표 간 **의존 구조**가 설계에 중요하다: 삼자 공동주의(9개월) → 가리키기 이해(12개월) → 단어-지시체 매핑 가속(12~18개월) → 어휘 폭발(18~24개월)은 선후 관계가 비교적 일관되며, 어머니 발화의 공동주의 정렬률(78% vs 50%)이 언어 결과와 연관된다는 점은 "공유 참조 프레임 확립 → 언어 접지"라는 순서를 하드코딩 대신 **전제조건 게이트**로 둘 근거다.
- 명시적 틀린 믿음(4세)과 DCCS(4.5~5세)의 동시성, 그리고 암묵 과제의 재현 실패는 "마음이론 모듈"보다 **작업기억·억제·표상 갱신** 같은 일반 자원의 성숙이 병목일 가능성을 높인다(단, Baillargeon 진영은 반론).
- 거울 자기인식의 문화 차(>50% vs <4%)는 "자기 표상"이 보편 시계가 아니라 **양육 맥락(자율성 지원·대면 상호작용 양)** 에 민감함을 뜻한다; 기계 유아에서 자기-모델 지표를 평가할 때 환경 의존성을 통제 변인으로 두어야 한다.

### Gaps
- Onishi & Baillargeon(2005)의 15개월 암묵적 틀린 믿음 원 논문은 직접 확인하지 않았고 복제 논쟁 문헌으로 간접 기술했다.
- 언어 이정표 수치는 Wikipedia 요약에 의존했으며 Wordbank/CDI 규준(퍼센타일별 연령)은 확보하지 못했다.

---

## 7. 얼마나 적은 데이터로 무엇이 학습되는가: 2~5세까지의 경험량과 아동 규모(child-scale) 학습 연구(SAYCam, BabyView, BabyLM, 2023~2026)

### Takeaway
아동은 초기 수년간 약 10^7~10^8 단어(LLM의 10^12~10^13 대비 4~5자릿수 적음)와 연간 수천 시간 수준의 시각 경험을 받는다. 단일 아동 헤드캠(SAYCam 61~221시간)으로도 범용 모델이 단어-지시체 매핑(4AFC 약 57%, 우연 25%)과 넓은 시각 범주(ImageNet 모델의 65~70%)를 학습하지만, 세밀한 객체중심성·의미/세계지식·합성성·약하게 정렬된 자연 입력의 활용(EgoBabyVLM 2026)·입력당 수익 가속(아동은 가속, LM은 일정) 등에서 아동에 못 미친다. BabyLM(1억 단어 이하)은 아키텍처·목적함수·커리큘럼 혁신으로 개선되지만 아동 수준에는 미달이며, 2025년에는 교사 피드백 "상호작용 트랙"이 추가되었다.

### Cited Findings
- 데이터 격차: 아동은 초기 발달 동안 10^7~10^8 단어를 받으며 LLM 학습 데이터(10^12~10^13 단어)보다 4~5자릿수 적다; 후보 설명은 기존 개념 지식, 멀티모달 접지, 상호작용적·사회적 입력 — [Frank 2023, TiCS](https://www.sciencedirect.com/science/article/abs/pii/S1364661323002036); [PDF](https://danielwharris.com/teaching/268/readings/Frank.pdf)
- 들리는 단어량: Hart & Risley(1995)는 전문직 2,153 / 노동계급 1,251 / 복지수급 가정 616 단어/시간을 관찰해 4년간 약 4,500만 대 1,300만 단어("3천만 단어 격차")를 외삽했다. Gilkerson et al.(2017)의 LENA 자동 분석 대규모 복제는 격차가 약 400만 단어에 가깝다고 했고, Sperry et al.(2018)은 주변 발화(ambient speech)를 포함하면 노동계급 공동체 추정치가 210% 증가함을 보였다 — [Wikipedia: Word gap](https://en.wikipedia.org/wiki/Word_gap)
- SAYCam: 3명 아동(S, A, Y), 6~31개월, 각 194/141/137시간(총 472시간), 5fps 약 900만 프레임 — [Orhan & Lake, arXiv HTML](https://arxiv.org/html/2305.15372). 아동 S 하위집합은 6~32개월 221시간이며, 37,486개 아동 지향 발화가 600,285 프레임과 짝지어진다 — [BabyCL 2026, arXiv HTML](https://arxiv.org/html/2606.05115)
- Orhan & Lake(2024): 단일 아동 200시간으로 ImageNet 모델의 65~70%(ImageNet 10% 학습 모델과 동등); 넓은 의미 범주·위치화 학습; 덜 객체중심·작은 객체 약함; "약 40일치 경험 vs 아동은 두 자릿수 더 많음" — [arXiv HTML](https://arxiv.org/html/2305.15372)
- Vong et al.(2024): 61시간, 6~25개월, 단어-지시체 매핑·제로샷 일반화 — [Science DOI](https://doi.org/10.1126/science.adi1374). BabyCL(2026)의 재현에서 오프라인 CVCL(400 에폭 셔플)은 Labeled-S 4AFC(우연 25%)에서 57.31±1.13%, 단일 시간순 패스 연속학습 BabyCL은 43.38±1.47%였다; 저자들은 오프라인과의 격차, 비디오 스트림 대비 큰 리플레이 버퍼, 합성성·일반화 벤치마크 부재를 한계로 명시 — [arXiv 2606.05115](https://arxiv.org/html/2606.05115)
- 견고성(2025): 세 아동·복수 아키텍처에서 접지 단어 학습 결과가 유지 — [Open Mind 2025](https://direct.mit.edu/opmi/article/doi/10.1162/OPMI.a.377/138563/On-the-Robustness-of-Modeling-Grounded-Word)
- 단일 아동 언어 입력 학습 가능성(2024): 6개 아키텍처 × 5개 데이터셋(단일 아동 3 + 기준 2)에서 통사·의미 단어 군집과 특정 언어 현상 민감성이 견고하게 형성됨 — [arXiv 2402.07899](https://arxiv.org/abs/2402.07899)
- Baby Scale(2026): BabyView 전사(6~36개월)로 학습한 LM은 문법 과제에서 수용 가능한 스케일링을 보이지만 의미·세계지식 과제에서는 합성 데이터 학습 모델보다 낮은 스케일링을 보였고, 아동 간 변동이 컸으며, 데이터 크기 외에 "분포적·상호작용적 언어 특징의 조합"이 성능과 가장 연관되었다(아동 언어 발달의 고품질 입력과 일치) — [arXiv 2603.29522](https://arxiv.org/abs/2603.29522)
- 수익 가속(2026): 아동은 입력 단위당 어휘 획득이 **가속**(각 단위에서 이전보다 더 많이 획득)하는 반면 아동 지향 발화로 학습한 LM은 스케일링 법칙대로 **일정한 비례 수익**을 보였다; 저자들은 "학습 입력의 점점 효율적인 사용"을 격차의 후보 설명으로 제시 — [arXiv 2608.17120](https://arxiv.org/abs/2608.17120)
- BabyView(2024): 868시간, 6개월~3세(및 취학 전 환경 5세까지), 고해상도·넓은 수직 시야 카메라, 자이로/가속도계, 전사·화자 분리·자세 추정 금표준 주석; 자기지도 언어·시각 모델의 전이 성능은 데이터 크기에 따라 증가하지만 큐레이션 데이터 학습보다 전반적으로 낮고 특히 시각에서 그렇다; "인간 학습자와 같은 데이터 규모·분포에서 인간 수준 성능을 어떻게 달성하는가"가 열린 과제 — [arXiv 2406.10447](https://arxiv.org/abs/2406.10447)
- EgoBabyVLM(2026): 영아·성인 자기중심 비디오 등 시각-언어 정렬 정도가 다른 데이터로 VLM을 학습·평가한 결과, "현재 VLM 패러다임은 큐레이션 데이터의 긴밀한 의미 정렬에 의존하며, 자연스러운 자기중심 입력을 지배하는 약하게 정렬된 신호 — 인간이 번성하는 바로 그 체제 — 를 활용하지 못한다"; 학습 어휘 빈도 구간별 어휘·문법 벤치마크 Machine-DevBench 제안 — [arXiv 2605.19130](https://arxiv.org/pdf/2605.19130)
- Zero-shot World Models(2026): 단일 아동의 1인칭 경험으로 학습한 희소·시간 인수분해 예측기(외형과 역학 분리) + 근사 인과추론 기반 제로샷 추정 + 합성적 추론이 여러 물리 이해 벤치마크에서 역량을 얻고, 학습 중 "점진적·단계적 역량 출현"을 보였다 — [arXiv 2604.10333](https://arxiv.org/abs/2604.10333)
- 소수 사례 일반화의 비밀(2025): 14명 영아의 87회 식사 헤드캠에서 8개 범주의 사례 분포는 극도로 치우쳐(같은 몇 객체의 많은 이미지 + 소수의 다른 사례) 있었고, 범주 경험은 "고유사·고변이의 덩어리진 혼합"으로 다수의 상호연결된 고유사 군집을 이뤘다; 이런 구조의 훈련 집합이 제한 데이터에서도 새 사례로의 일반화를 지원했다 — [arXiv 2510.15060](https://arxiv.org/abs/2510.15060)
- 커리큘럼(Sheybani 2023): 가장 어린 월령 데이터부터 학습하는 것이 최선(5절 참조) — [NeurIPS 2023](https://proceedings.neurips.cc/paper_files/paper/2023/file/a9ad92a81748a31ef6f2ef68d775da46-Paper-Conference.pdf)
- 규모 확장(HVM-1, 2024): Ego4D(74%)·SAYCam 등 6개 데이터셋 약 5,000시간의 인간형 자기중심 비디오로 사전학습한 모델은 448 해상도에서 Kinetics-700 학습 모델과 경쟁적이었다; 저자들은 5,000시간이 "약 반년의 시각 경험(하루 12시간 수면을 감안하면 1년 남짓)"에 해당하며 아동은 10~20년에 걸쳐 한 자릿수 이상 더 많은 경험을 쌓는다고 했다 — [arXiv 2407.18067](https://arxiv.org/html/2407.18067v1)
- BabyLM 2024(2차 대회): 31개 제출 중 하이브리드 인과-마스크 LM 아키텍처가 최고였고, 멀티모달 트랙에서는 기준선을 넘은 제출이 없었다; 하이브리드 아키텍처·훈련 데이터 수정(구조화된 변이 집합 주입)·목적함수 혁신이 고정 데이터 체제에서 측정 가능한 개선을 냈으며, 총 훈련 FLOPs와 평균 성능 간 강한 관계가 관찰되었다; 1억 단어 미만 모델이 1천만 단어 미만 모델을 능가하지만 아동 수준·LLM 수준에는 미달 — [Findings of the Second BabyLM Challenge, arXiv 2412.05149](https://arxiv.org/pdf/2412.05149); [요약](https://www.emergentmind.com/topics/2nd-babylm-challenge)
- BabyLM 2025(3차 대회): 텍스트, 텍스트-이미지, 그리고 모델이 교사 피드백으로 학습하는 새 "상호작용 트랙"으로 확장; 새 목적함수·아키텍처가 가장 강했고, FLOPs와 성능 사이에 완전한 상관은 없었다; strict-small 우승은 훈련 중 마스킹 분포를 적응시키는 AMLM hard decay; 단순한 텍스트는 언어 지식 과제에, 복잡한 텍스트는 세계지식·개체 추적 과제에 유리 — [ACL Anthology BabyLM 2025](https://aclanthology.org/events/babylm-2025/); [요약](https://www.emergentmind.com/topics/babylm-challenge)
- 영아 학습에서 비지도 ML로의 교훈(Zaadnoordijk, Besold & Cusack, Nature Machine Intelligence 2022): 영아 인지 발달과학이 차세대 비지도 학습의 열쇠일 수 있다며 영아 학습의 질과 속도를 가능케 하는 요인들을 정리 — [arXiv 2009.08497](https://arxiv.org/abs/2009.08497); [NMI 2022 DOI](https://doi.org/10.1038/s42256-022-00488-2)
- 관련 2024~2025 제목(내용 미확인): "Human Gaze Boosts Object-Centered Representation Learning"(2025), "Toddlers' Active Gaze Behavior Supports Self-Supervised Object Learning"(2024), "Do Blind Spots Matter for Word-Referent Mapping?"(2025), "BabyVLM: Data-Efficient Pretraining of VLMs Inspired by Infant Learning"(2025), "Is Child-Directed Speech Effective Training Data for Language Models?"(2024) — [arXiv 2501.02966](https://arxiv.org/pdf/2501.02966); [arXiv 2411.01969](https://arxiv.org/pdf/2411.01969); [arXiv 2511.11725](https://arxiv.org/pdf/2511.11725); [arXiv 2504.09426](https://arxiv.org/pdf/2504.09426); [arXiv 2408.03617](https://arxiv.org/pdf/2408.03617)

### Inferences
- 시각 경험량 추정(추론; 직접 출처 없음): HVM-1 저자의 가정(하루 12시간 각성)을 쓰면 연간 약 4,400시간, **2세까지 약 8,800시간, 5세까지 약 22,000시간**이다(신생아는 수면이 더 길어 실제는 다소 적다). 이에 비해 SAYCam-S 221시간은 6~32개월 각성 시간의 약 2~3%에 불과하며, 그럼에도 ImageNet 모델의 65~70%와 4AFC 57%를 얻은 것이 현 "하한선"이다.
- 아동-모델 격차의 **성격**이 2025~2026 연구에서 구체화되고 있다: (1) 입력 정렬 의존(EgoBabyVLM) — 아동은 약하게 정렬된 입력에서도 배운다, (2) 수익 가속(2608.17120) — 아동은 학습이 학습을 돕는다(learning-to-learn/부트스트래핑), (3) 의미·세계지식의 낮은 스케일링(Baby Scale) — 텍스트 통계만으로는 부족하고 접지·상호작용이 필요, (4) 단일 패스 연속학습의 손실(BabyCL 57→43%) — 아동은 리플레이 없이(또는 수면 중 재생으로) 시간순 학습을 한다.
- 기계 유아에 바로 쓸 수 있는 데이터 설계 원칙: 가장 느리고 단순한 초기 입력부터(Sheybani), 소수 객체의 고반복·고변이 덩어리 분포(2510.15060), 아동 주시 대상에 시간 정렬된 발화(공동주의 78%), 교사 피드백 루프(BabyLM 2025 상호작용 트랙).

### Gaps
- Vong 2024 원문의 수치(Labeled-S 정확도, Konkle 새 객체 일반화 정확도, 각성 시간 대비 비율)는 PDF 파싱 실패로 2026년 재현 논문의 수치로 대체했다; 원 논문 수치와 다를 수 있다.
- BabyLM 2024 토큰 예산은 한 요약에서 "strict 1억 / strict-small 1천만 단어"로, 다른 자동 요약에서는 상이하게 보고되어 상충했다; 1억/1천만이 통용 수치이나 원문 확인 필요.
- 아동의 **청각** 경험 총량(시간)·각성 시간의 연령별 규준은 직접 출처를 확보하지 못했다.
- Zaadnoordijk et al.의 "다섯 가지 교훈"의 구체 목록은 초록이 제공되지 않아(한 요약은 "세 가지 요인"이라 표기해 상충) 기술하지 못했다.

---

## 8. 설계 함의: 인공 발달 에이전트에서 무엇을 하드코딩하고 무엇을 학습시킬 것인가(후보 목록)

### Takeaway
증거가 수렴하는 결론은 "지식을 하드코딩하지 말고 **배선 규칙·가치 신호·주의 편향·발달 스케줄**을 하드코딩하라"는 것이다(Zador의 게놈 병목, Panksepp의 1차 정서, Johnson & Morton의 CONSPEC, Gergely & Watson의 수반성 스위치, Hensch의 브레이크). 개념 수준 지식(객체 범주, 단어 의미, 직관 물리의 세부, 마음이론)은 현재 증거상 적절한 입력·상호작용·호기심 보상이 있으면 학습 가능 범위에 있으며, 논쟁 영역(객체중심성, 합성성, 수 감각의 정확한 선천성)은 "약한 편향 + 학습"으로 설계하고 절제(ablation)로 검증하는 것이 안전하다.

### Cited Findings
- 하드코딩 근거(배선 규칙·아키텍처): 게놈은 가중치가 아닌 배선 규칙을 부호화하며, 선천 회로(장소세포 성향·얼굴 회로) 위에 특정 내용이 학습된다 — [Zador 2019](https://www.biorxiv.org/content/10.1101/582643v1.full)
- 하드코딩 근거(가치/정서): 7개 1차 정서 시스템은 신피질 없이 작동하는 선천 피질하 회로 — [Frontiers 2019](https://www.frontiersin.org/journals/neuroscience/articles/10.3389/fnins.2018.01025/full)
- 하드코딩 근거(주의 편향): 신생아 얼굴 정향은 피질하 CONSPEC이 담당해 피질 학습의 입력을 편향시킨다 — [Johnson 2005](https://www.researchgate.net/publication/7492601_Subcortical_face_processing_Review)
- 하드코딩 근거(수반성 탐지와 단일 파라미터 전환): 완벽→불완전 수반성 선호 전환(≈3개월) — [Gergely 2001](https://pubmed.ncbi.nlm.nih.gov/11531136/)
- 하드코딩 근거(내재 보상 형식): 학습 진전(오차 감소율)을 영역별로 추적하는 보상이 발달 궤적을 자기조직 — [Oudeyer & Kaplan 2007](https://www.frontiersin.org/articles/10.3389/neuro.12.006.2007/full); 기대 위반이 학습·탐색을 증폭 — [Stahl & Feigenson 2015](https://doi.org/10.1126/science.aaa3799)
- 하드코딩 근거(가소성 스케줄): PV+ 성숙과 분자 브레이크가 영역별 결정적 시기를 열고 닫는다 — [Hensch](https://www.esforum.de/publications/PDFs/sfr25/SFR25_05_Hensch.pdf)
- 학습 가능 근거(시각 범주·위치화): 단일 아동 200시간, 강한 귀납 편향 없이 ImageNet 모델의 65~70% — [Orhan & Lake 2024](https://arxiv.org/html/2305.15372)
- 학습 가능 근거(단어 접지): 61시간, 범용 대조 학습으로 단어-지시체 매핑과 제로샷 일반화 — [Vong et al. 2024](https://doi.org/10.1126/science.adi1374)
- 학습 가능 근거(A-not-B 같은 "단계 행동"): 자세·지연·현저성의 역학에서 창발 — [Wikipedia: A-not-B](https://en.wikipedia.org/wiki/A-not-B_error)
- 미해결/약한 편향 권장 근거: 객체중심 편향 필요 여부는 열린 질문 — [Orhan & Lake 2024](https://arxiv.org/html/2305.15372); 합성성·일반화 벤치마크 부재 — [BabyCL 2026](https://arxiv.org/html/2606.05115); 직관 물리·심리는 "start-up software"로 넣어야 한다는 입장 — [Lake et al. 2017](https://www.cs.princeton.edu/~bl8144/papers/LakeEtAl2017BBS.pdf); 핵심 지식을 합성 데이터로 주입하자는 2025 제안 — [arXiv 2502.10742](https://arxiv.org/abs/2502.10742)
- 상호작용 필수 근거: 동일 내용의 비디오/오디오로는 음소 학습이 일어나지 않음 — [Kuhl 2003(2013 리뷰 기술)](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2013.00718/full); BabyLM 2025 상호작용 트랙 — [ACL Anthology](https://aclanthology.org/events/babylm-2025/)
- 커리큘럼 근거: 어린 월령 데이터 우선 — [Sheybani 2023](https://proceedings.neurips.cc/paper_files/paper/2023/file/a9ad92a81748a31ef6f2ef68d775da46-Paper-Conference.pdf); 입력의 덩어리진 고반복·고변이 구조 — [arXiv 2510.15060](https://arxiv.org/abs/2510.15060)

### Inferences
- **하드코딩 1순위(증거 강함)**: (a) 항상성·정서 가치 채널(SEEKING류 탐색 음조, 분리/위협 벌점, 사회적 접촉·놀이 보상); (b) 주의 편향(얼굴/생물학적 운동/소리 발원지 정향, 움직임·대비); (c) 자기 행동-결과 수반성 탐지기와 그 선호값의 시간 스케줄(완벽→불완전); (d) 영역별 학습 진전 추적에 기반한 내재 보상과 기대 위반 시 탐색 증폭; (e) 모듈/영역별 가소성 창(학습률·정규화·브레이크) 스케줄; (f) 초기 고정 행동 루틴(반사)과 학습 정책에 의한 억제 메커니즘; (g) 센서·행동 해상도의 점진적 해제(신체 발달 모사).
- **약한 편향으로 넣고 절제 실험으로 검증(논쟁 영역)**: 객체 단위 분절(objectness)·시간적 지속성 prior, 근사 수량 채널, 행위자/목표 탐지(자기추진 운동·수반적 반응성에 대한 민감성), 공간 기하 prior. Spelke 진영은 이를 "핵심 지식"으로, Orhan & Lake/Smith & Thelen 진영은 "학습 가능"으로 보며, 현재 증거는 둘 다 완전히 배제하지 못한다.
- **학습시킬 것(증거 강함)**: 시각 범주와 위치화, 단어-지시체 매핑, 환경 특이적 지각 전문화(협소화), 사회 파트너의 개체 식별과 애착 대상 선택, A-not-B 같은 과제 행동, 명시적 틀린 믿음·실행기능(일반 자원 성숙과 함께), 자기 모델(거울 자기인식은 양육 맥락 의존).
- **환경/교사 측 설계**: 반응형(수반적) 교사 에이전트, 아동 주시 대상에 정렬된 발화, ZPD 기반 난이도 조절(학습 진전 신호를 교사도 공유), 시간순 입력을 허용하되 수면 유사 리플레이 단계로 연속학습 손실 보완(BabyCL 격차 참고), 어린 단계일수록 느리고 단순한 입력.
- **평가 설계**: 단일 과제 통과/실패 대신 다중 조건(지연·자세·맥락) 수행 지형을 보고하고, 응시형(암묵) 지표와 행동형(명시) 지표를 분리 기록한다; 암묵적 마음이론의 재현 실패는 암묵 지표만으로 "이해"를 선언하지 말라는 경고다.

### Gaps
- "약한 편향 + 학습"과 "강한 핵심 지식 주입"을 동일 데이터·동일 벤치마크에서 직접 비교한 2024~2026 절제 연구는 이번 조사에서 찾지 못했다(Orhan & Lake의 결론은 편향 없는 조건의 성능 하한만 제공).
- 정서 시스템(Panksepp)을 보상 채널로 구현했을 때 학습 효율·안전성에 미치는 영향에 대한 정량 연구는 확보하지 못했다.
- 인간 영아의 각성 시간·청각 입력 총량의 연령별 규준, 그리고 BabyLM 토큰 예산·Vong 원 수치는 원전 확인이 필요하다(7절 Gaps 참조).
