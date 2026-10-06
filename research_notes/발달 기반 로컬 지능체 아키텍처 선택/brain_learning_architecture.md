# 뇌의 학습 아키텍처: 저전력·지속학습 에이전트를 위한 시스템/계산 신경과학 청사진

> 작성일 2026-10-06. 범위: "20W로 평생 학습하는 뇌"의 조직 원리를 (1) 확립된 신경과학, (2) 계산 이론, (3) AI 구현 결과로 분리해 정리하고, 소비자 PC/소형 장치에서 소프트웨어로 옮길 수 있는 설계 원칙을 추출한다.
> 표기: **[확립]** = 다수 실험으로 뒷받침되는 신경과학적 사실, **[이론]** = 계산 이론/가설, **[논쟁]** = 핵심 신경과학자들 사이에서 다투는 중, **[AI 미검증/실패]** = AI로 옮겼을 때 이득이 재현되지 않았거나 규모 확장이 안 된 경우.
> 주의: 세션의 WebSearch 예산이 소진되어 일부 주제(예: Hassabis 2017 본문, Kumaran 2016 본문, 편도체 선천 회로, Hinton FF 본문의 수치)는 초록·2차 요약 수준에서만 확인했다. 해당 항목은 각 질문의 Gaps에 명시했다.

---

## 1. 선천(hardwired) vs 학습(plastic): 무엇이 유전체에 박혀 있고 무엇이 경험으로 채워지는가

### Takeaway
뇌의 "배선 규칙"(뇌간·시상하부 본능 회로, 기저핵 루프, 피질 정준 미세회로, 시상피질 루프의 레이아웃)은 유전체가 압축된 생성 규칙으로 지정하고, 피질의 **내용**(수용장·지도·개념)은 입력 통계로 채워진다 [확립]. Zador의 "genomic bottleneck" 논증은 유전체가 연결체를 직접 지정할 수 없음을 정량적으로 보이며(≥5~6 자릿수 부족), 2024년 PNAS 후속 연구는 1,000배 압축된 "유전체 네트워크"가 학습 전 성능 79~94%를 낼 수 있음을 보여 "선천 prior = 압축된 배선 생성기"라는 설계 원리를 뒷받침한다.

### Cited Findings
- **Genomic bottleneck 정량 논증.** 인간 유전체 ≈ 3×10⁹ 뉴클레오타이드(≈1GB), 뇌 ≈10¹¹ 뉴런 × >10³ 시냅스, 연결 하나 지정에 ≈37비트 → 3.7×10¹⁵비트 필요, 유전체 용량 대비 "at least six orders of magnitude too small". 따라서 "the genome cannot specify the explicit wiring diagram, but must instead specify a set of rules for wiring up the brain during development" — [Zador 2019, Nat Commun (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6704116/)
- **대부분의 동물 행동은 선천적.** 설치류 후각 반응, 생쥐의 looming 탐지, 사슴쥐 굴파기 행동이 "independent of parenting"으로 일정; "most of the behavioral repertoire of insects and other short-lived animals is innate". 해마 place cell도 "innate propensity but learned content", 얼굴 선호는 선천이나 특정 얼굴 인식은 학습 — [Zador 2019](https://pmc.ncbi.nlm.nih.gov/articles/PMC6704116/)
- **진화 = 외부 루프, 학습 = 내부 루프.** "supervised learning in ANNs should not be viewed as the analog of learning in animals"; 대신 "supervised evolution"이라 부르자고 제안. 동물은 "two nested optimization processes: an outer 'evolution' loop ... and an inner 'learning' loop"를 가짐. 유전체 크기 제한은 "may act as a regularizer or an information bottleneck, shifting the balance from variance to bias" — [Zador 2019](https://pmc.ncbi.nlm.nih.gov/articles/PMC6704116/)
- **압축된 배선 생성기 실험(g-network).** 뉴런마다 이진 레이블(log₂N 비트)을 주고 (전·후 시냅스 레이블 쌍) → 가중치를 출력하는 작은 "genomic network"로 전체 네트워크를 손실 압축. MNIST 2층 MLP(6×10⁵ 파라미터): 322배 압축 시 초기(학습 전) 정확도 94%(완전 학습 98%), 1,038배 압축 시 79%. CIFAR-10 9층 CNN(1.4×10⁶ 가중치): 92배 압축 시 76%(완전 학습 89%). CIFAR-10→SVHN 전이학습에서 92배 적은 파라미터만 옮기고도 표준 전이학습과 동등, 특히 하위 층의 전이가 좋아져 bottleneck이 "generalizable structure"를 추출한다고 해석 — [Shuvaev, Lachi, Koulakov, Zador 2024 PNAS (bioRxiv 전문)](https://www.biorxiv.org/content/10.1101/2021.03.16.435261v1.full); [PubMed](https://pubmed.ncbi.nlm.nih.gov/39264740/)
- **시상하부 본능 회로는 하드와이어드.** 성·양육·공격 행동은 "hardwired neural circuits"가 지원; 내측 시삭전핵(MPN)·VMHvl·PMv로 구성된 reproductive behavior control column(RBCC)이 호르몬·대사 신호를 통합하고 중뇌변연 도파민계를 동원해 사회적 동기를 유지, RBCC와 뇌간이 "dual-control system"으로 순간순간 행동을 생성 — [Mei, Osakada, Lin 2023 Science](https://www.science.org/doi/10.1126/science.adh8489)
- 그러나 최근 대규모 기록은 시상하부 행동 코딩이 "more high-dimensional, spatially distributed, dynamic, and variable than previously appreciated"임을 보여 "하드와이어드 ≠ 단순 스위치" — [Anatomically distributed neural representations of instincts in the hypothalamus (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC10690204/); 사회행동 회로는 "innate yet flexible" — [Neuron 2021](https://www.sciencedirect.com/science/article/pii/S0896627321001124)
- **피질 내용은 입력이 결정(재배선 실험).** 족제비에서 시각 입력을 청각 경로로 돌리면 "재배선된 청각피질"에 V1과 유사한 방향 선택성 모듈이 형성(단, V1보다 덜 질서정연). "afferent activity has a profound influence on diverse components of cortical circuitry, including thalamocortical and local intracortical connections" — [Sharma, Angelucci, Sur 2000 Nature](https://doi.org/10.1038/35009043)
- **정준 피질 미세회로(canonical microcircuit)의 보존.** 시상 입력이 L4로 들어와 L2/3→L5/6으로 흐르고 L6→L4로 루프를 닫는 구조적 모티프가 "shared across cortical areas and found in multiple mammalian species"; 시상 입력은 어떤 뉴런에도 주된 흥분원이 아니고 "intracortical excitatory connections provide most of the excitation"(강한 재귀성) — [Douglas & Martin, A Canonical Microcircuit for Neocortex](https://www.researchgate.net/publication/242918941_A_Canonical_Microcircuit_for_Neocortex); 피질-시상·피질-피질 루프 모티프와 층별 연결 패턴이 여러 영역에서 일관 — [Frontiers Neural Circuits 2022](https://public-pages-files-2025.frontiersin.org/journals/neural-circuits/articles/10.3389/fncir.2022.972157/xml/nlm); 미세회로를 "architectural bias"로 보는 계산 연구 — [Neural Computation 2025](https://direct.mit.edu/neco/article/37/9/1551/131940/Exploring-the-Architectural-Biases-of-the-Cortical)
- **AI 프레임에서의 위치.** Richards et al.은 뇌 이해의 세 설계 요소를 objective functions / learning rules / architectures로 나누고, 진화가 제공하는 "strong inductive biases"가 하드와이어드 행동까지 형성한다고 명시 — [Richards et al. 2019 Nat Neurosci (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7115933/)
- Hassabis et al.은 "better understanding biological brains could play a vital role in building intelligent machines"라는 입장에서 신경과학→AI 영향사를 정리 — [Hassabis et al. 2017 Neuron](https://doi.org/10.1016/j.neuron.2017.06.011)

### Inferences
- 설계 원리 1 — **선천 스캐폴드와 학습 내용을 분리하라.** 뇌는 "무엇을 배울지"가 아니라 "어떤 회로 레이아웃과 어떤 학습 신호로 배울지"를 유전체에 넣는다. 소프트웨어에서는 (a) 모듈 토폴로지·루프 구조·학습 신호 배선은 손으로(또는 외부 루프 최적화로) 고정, (b) 각 모듈 내부 가중치만 온라인 학습으로 채우는 2단 구조가 대응물이다.
- 설계 원리 2 — **"학습 전 성능(innate performance)"을 설계 지표로 삼아라.** Shuvaev 2024의 g-network처럼, 압축된 생성기(하이퍼네트워크/규칙)에서 초기화된 네트워크가 0-step에서 얼마나 하는지가 prior의 품질 척도다. 이는 전이·few-shot의 기반이 된다.
- 설계 원리 3 — **정준 미세회로 = 반복 가능한 모듈 템플릿.** 동일한 블록(입력층→표층 처리→심층 출력→루프백)을 복제하고, 입력만 바꾸면 내용이 달라진다는 Sur의 재배선 결과는 "모듈을 범용 템플릿으로 찍어내고 데이터로 전문화한다"는 소프트웨어 설계를 정당화한다.
- 주의: "innate"는 고정 룩업테이블이 아니다. 시상하부조차 분산·동적 표현을 쓴다. 본능 모듈도 상태 의존적 파라미터화(호르몬·대사 신호 입력)를 가져야 한다.

### Gaps
- 유전체 중 **어느 비율**이 배선 지정에 쓰이는지에 대한 정량치는 찾지 못했다(Zador는 "필요량 대비 자릿수 부족"만 논증). 뇌 발현 유전자 비율 같은 수치는 이 세션에서 확인 불가.
- 편도체의 선천 공포 회로(looming·포식자 냄새 등)에 관한 1차 문헌은 검색 예산 소진으로 수집하지 못했다. Zador 2019의 looming 탐지 언급만 확보.
- Hassabis 2017·Richards 2019는 초록/PMC 요약 수준에서 확인했고 본문 세부 인용은 제한적이다.

---

## 2. 예측처리(Predictive Processing)/예측부호화(Predictive Coding): 기전 주장, 증거, 가장 강한 비판, 정밀도(precision)=주의

### Takeaway
PC의 기전 주장은 "각 위계 수준에 예측 뉴런과 오차 뉴런이 따로 있고, 하향 예측이 상향 오차를 상쇄하며, 오차는 정밀도(역분산)로 가중된다"는 것이다 [이론]. 기대 억제·omission 반응·예측 지연 위계 등은 잘 재현되지만 [확립], "오차 전용 뉴런 집단의 직접 증거", "음의 오차 표현", "정밀도의 생물학적 구현"은 미해결이고 적응(adaptation)만으로 상당 부분 설명 가능하다는 비판이 강하다 [논쟁]. 정밀도=주의 이론(Feldman & Friston)은 우아하지만 N² 정밀도 행렬을 시상이 다룰 수 없다는 규모 문제가 지적된다.

### Cited Findings
- **기전 주장.** "prediction errors ϵl are computed in the superficial layers at the level below, and then transmitted...to the superficial layers"의 value(예측) 뉴런 μ로; 다중 스케일 예측("from fine details ... at low levels, to global invariant properties ... at higher levels"); 정밀도(Σ⁻¹)가 오차를 곱셈적으로 조절하며 "can be interpreted as implementing top-down attentional modulation" — [Millidge, Seth, Buckley, PC review (ar5iv)](https://ar5iv.labs.arxiv.org/html/2107.12979)
- **지지 현상.** 말단정지(end-stopping) 뉴런의 extra-classical RF 효과("fired if a bar ... ended just outside the receptive field ... but not if it continued"), 반복 억제, Kanizsa 착시 윤곽에 대한 V1 반응 — [Millidge et al.](https://ar5iv.labs.arxiv.org/html/2107.12979)
- **Walsh et al. 2020의 체계적 평가.** PP의 4대 검증 가능 주장: (1) 오차 반응은 기대와 반비례, (2) 하향 신호 = 감각 예측, (3) 각 수준에 예측/오차 두 기능적 하위집단 존재, (4) 수준 간 오차·예측 교환으로 오차 최소화. **지지됨**: omission 반응, 확률로 조절되는 반복 억제, 다중 기록법에서의 기대 억제, 위계적 지연 차이, 운동 피드백에 의한 V1 예측 활동. **약함/모호**: "a significant contingent failed to replicate these effects"; 침습/비침습 기록 간 불일치; 조절이 "specific to individually predicted exemplars". **핵심 비판**: 적응 혼입(Vinken & Vogels는 자극 특이 적응만으로 기대 억제를 재현), fMRI/MEG는 오차·예측 집단을 합산해 "magnitudes of their relative contributions are not easily derived a priori", 기능적 분리 집단의 직접 증거는 제한적(Fiser et al.의 V1 단일 유닛 연구 정도), 베타/감마 분리 예측은 "notable discrepancies", 자극 확률·주의·훈련 길이 조합마다 PP의 예측이 달라져 반증 기회가 줄어듦. 결론: 기대가 감각 처리에 영향을 준다는 "strong evidence"는 있으나 이것이 진짜 PP인지는 "equally plausible" 대안(적응)과 미분리 — [Walsh, McGovern, Clark, O'Connell 2020 Ann NY Acad Sci (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7187369/)
- **Millidge 리뷰의 자체 미해결 목록.** (a) 오차 뉴런 식별: "It is not clear whether such neurons are prediction errors...or whether they represent value neurons"; (b) "Neurons cannot have a negative firing rate" → 양/음 오차 집단 분리를 가정하나 1:1 연결 증거 없음; (c) 정밀도: "It is unlikely that the pulvinar could usefully process or precisely modulate this N² amount of information"; (d) 심층(deep layer)이 모델에서 예측 중계 외 역할이 없어 피질하 투사가 설명되지 않음; (e) 표층 감마 vs 심층 알파/베타 주파수 분리는 미세회로 모델에서 자연히 도출되지 않음; (f) 피질-피질 피드백이 흥분성인데 PC는 억제성 예측을 요구; (g) 엄격한 위계 가정 vs 실제의 skip/측면 연결; (h) rate code 가정("if the brain does use spiking-codes heavily, then ... would need to be reformulated"); (i) "Predictive coding does not actually make clear predictions of the level of activity in whole brain regions" — [Millidge et al.](https://ar5iv.labs.arxiv.org/html/2107.12979)
- **PC ≈ backprop 조건.** 예측 고정·정밀도=단위행렬·동역학 수렴 시 value 뉴런 업데이트가 backprop 기울기와 일치하고, 정밀도를 학습하면 "predictive coding forms a superset of backpropagation which allows it to weight gradients by their intrinsic variance"(자연 기울기와 유사). 그러나 오차가 θᵀ로 전달되어야 하므로 "a considerable issue of weight transport" 존재 — [Millidge et al.](https://ar5iv.labs.arxiv.org/html/2107.12979); 원 결과: "under certain parameters, the weight modifications in their predictive coding model align with those of the backpropagation algorithm" — [Whittington & Bogacz 2017 Neural Computation](https://doi.org/10.1162/NECO_a_00949)
- **정밀도 = 주의.** 주의란 "inferring the level of uncertainty or precision during hierarchical perception"; 정밀도는 "the synaptic gain (post-synaptic responsiveness) of units reporting prediction errors"로 구현되며 감마 동기화와 아세틸콜린("acetylcholine appears to increase synaptic gain directly by ... reducing spike-frequency adaptation")이 후보. Posner 단서 과제 시뮬레이션에서 유효 단서 시 ~20ms 반응 이득, "biased competition emerges naturally in Bayes-optimal schemes" — [Feldman & Friston 2010 Front Hum Neurosci](https://www.frontiersin.org/articles/10.3389/fnhum.2010.00215/full)
- **대안 구현 제안.** 오차를 별도 뉴런이 아닌 피라미드 뉴런의 수상돌기에서 계산한다는 제안 — [Where is the error? Dendritic error computation, TINS 2022](https://www.cell.com/trends/neurosciences/fulltext/S0166-2236(22)00186-2); PP의 반증가능성 논쟁 — [Testable or bust, Synthese 2022](https://link.springer.com/article/10.1007/s11229-022-03891-9); 청각 시상에서 "부재(absence)" 전용 예측오차 신호 보고(2025 preprint) — [arXiv 2511.21605](https://arxiv.org/pdf/2511.21605)
- **2024년 "prospective configuration".** PC형 에너지 기반 네트워크에서 가중치 변경 전에 활동이 먼저 목표 상태로 수렴하는 학습 원리가 "more efficient and effective in many contexts faced by biological organisms"(온라인·지속학습 등)이고 인간·쥐 실험 패턴을 재현한다고 보고 — [Song, Millidge, Salvatori, Lukasiewicz, Xu, Bogacz 2024 Nat Neurosci](https://www.nature.com/articles/s41593-023-01514-1)
- **ML 구현 현황 [AI 미검증/실패].** PCN은 inference learning(반복 추론 후 가중치 갱신)으로 훈련되며 CIFAR-10 수준 결과만 보고, "performance degrades significantly on larger datasets beyond small benchmarks", ImageNet 규모 성공 사례 없음, 수렴 불안정·속도 문제·에너지 이득 미입증 — [Introduction to PCNs for ML 2025](https://arxiv.org/pdf/2506.06332); PCX 벤치마크 라이브러리(CIFAR-10/100, Tiny ImageNet)는 PC의 SOTA를 갱신하면서도 "scalability as a major open problem"으로 명시 — [Pinchetti et al., Benchmarking PCNs (PCX)](https://arxiv.org/abs/2407.01163)

### Inferences
- 엔지니어 관점에서 PC의 **안전한 부분**은 (1) 하향 예측과 상향 오차의 분리된 채널, (2) 오차에 대한 이득(정밀도) 제어, (3) 기대 억제로 인한 통신량 절감이다. 이 셋은 신경과학적 증거와 무관하게 소프트웨어에서 유용하고 저비용이다. **위험한 부분**은 "PC로 backprop을 대체해 규모를 키운다"는 기대 — 2026년 현재 ImageNet급 성공 사례가 없다.
- 정밀도=주의는 구현 시 "채널별 스칼라 이득"(대각 정밀도)으로 축소해야 현실적이다. Millidge 리뷰의 N² 비판은 뇌에도 소프트웨어에도 같은 교훈: 전체 공분산이 아니라 저차원 이득 벡터를 신경조절 신호(ACh/NE 유사)로 조정하라.
- Walsh의 "적응 혼입" 비판은 설계상 이득이다: 단순 적응(지수 평균 빼기)만으로도 기대 억제의 상당 부분을 얻는다. 복잡한 생성 모델 이전에 **정규화·적응 필터부터** 넣는 것이 비용 대비 효율적이다.
- prospective configuration은 "가중치 전에 활동을 먼저 조정"하는 에너지 기반 추론이 지속학습에 유리할 수 있음을 시사하나, 대규모 검증은 없다 — 소형 모듈(수천~수만 파라미터)에서 실험할 가치는 있다.

### Gaps
- PC가 ImageNet 규모에서 backprop 대비 어느 정도 정확도인지 **구체 수치**를 1차 문헌에서 확인하지 못했다(PCX 초록은 수치 미제공).
- Rao & Ballard 1999 원 논문, Clark/Hohwy의 철학적 정식화는 직접 인용하지 못했고 Millidge·Walsh 리뷰를 통해 간접 확인했다.

---

## 3. 자유에너지 원리(FEP)와 능동추론(Active Inference): 구현(pymdp, deep AIF)과 2026년 기준 규모 한계

### Takeaway
능동추론은 "행동 선택을 베이즈 추론으로 보는 통일 언어"로서 가치가 있으나, 핵심 개발자 중 한 명(Millidge)의 회고에 따르면 신경망으로 규모를 키우는 순간 표준 RL과 사실상 동형이며, 기대자유에너지(EFE)의 탐색 이점은 "가정에서 결론을 끌어낸 것"으로 실무적 우위가 없다 [논쟁/AI 미검증]. pymdp는 이산 상태공간에서 손으로 짠 A/B 행렬을 요구해 조합 폭발 한계가 있고, 2025년 VERSES의 AXIOM은 객체중심 prior+베이즈 모델 축소로 Gameworld 10k에서 DreamerV3를 앞섰다고 주장하나(회사 보도자료 수치), 이는 "강한 구조적 prior"의 승리이지 FEP 일반의 승리로 보기 어렵다.

### Cited Findings
- **Millidge의 회고(2024).** "the most appealing and interesting thing about active inference is that it shines light on relating and understanding action selection as a Bayesian inference problem"; 신경망으로 분포를 파라미터화하자 "results were generally successful but lead to the immediate question of how active inference differs from standard RL"; AIF와 RL은 다른 보상 인코딩을 가진 베이즈 추론으로 상호 재정식화 가능하며 그 구분은 "little practical importance for designing action selection algorithms"; EFE는 "not directly derivable from a standard Bayesian treatment"이고 기존 유도는 "largely assumed their conclusion"; 실무적으로 "The Bayesian lens is much less valuable" — [Beren Millidge, A Retrospective on Active Inference](https://www.beren.io/2024-07-27-A-Retrospective-on-Active-Inference/)
- **pymdp의 한계.** POMDP용 라이브러리로 손으로 설계한 확률 행렬이 필요해 상태공간이 커지면 조합적으로 폭발; 격자세계·T-미로는 되지만 실세계 복잡도는 안 됨(검색 요약) — [pymdp 논문 (ar5iv)](https://ar5iv.labs.arxiv.org/html/2201.03904)
- **Deep active inference.** 신경망 근사로 확장성은 얻지만 원칙적 베이즈 성격을 잃고, 복잡 과제에서 순수 RL을 넘지 못했으며 표현이 불투명(검색 요약); "Since all current approaches tackle low-dimensional problems, it is unclear how they would scale up to more complex domains" — [Mazzaglia et al., FEP for perception and action: a deep learning perspective](https://arxiv.org/pdf/2207.06415); MC 기반 deep AIF — [Fountas et al. 2020](https://arxiv.org/pdf/2006.04176)
- **FEP 자체에 대한 철학적 비판.** Markov blanket의 베이즈망적(인식적) 사용과 FEP의 형이상학적 사용 사이에 "a persistent confusion"이 있어 "Pearl blankets"와 "Friston blankets"를 구분해야 하며, 후자는 수학적 타당성을 넘어선 추가 철학적 가정을 요구 — [Bruineberg et al., The Emperor's New Markov Blankets, BBS](https://doi.org/10.1017/S0140525X21002351)
- **AXIOM(2025).** 객체중심 혼합모델 + 조각별 선형 동역학 + 온라인 확장 + 주기적 Bayesian model reduction + 최소 기울기 최적화로 "masters various games within only 10,000 interaction steps", DRL 대비 훨씬 적은 파라미터 — [AXIOM arXiv 2505.24784](https://arxiv.org/abs/2505.24784). 회사 발표 수치: Gameworld 10k에서 DreamerV3 대비 정규화 점수 77 vs 48(60% 우위), 3,175 vs 24,207 스텝(7.6배 샘플효율), GPU 시간 ~10분 vs ~370분(39배), 비용 $0.66 vs $25.54 — [VERSES 보도자료](https://www.verses.ai/news/verses-digital-brain-beats-googles-top-ai-at-gameworld-10k-atari-challenge) (1차 검증은 회사 측 주장; arXiv 초록은 구체 비교 수치 미제시)
- **2026년 "test-time scaling" 프레임.** 자율주행 과제에서 소프트 베이즈 추론으로 테스트 시 정책을 갱신, 모델프리 Q-learning·베이즈 RL 대비 "robust generalization to unforeseen scenarios while improving inference efficiency by over 36%" 주장(53쪽, 변분 근사 필요) — [arXiv 2606.22813](https://arxiv.org/abs/2606.22813)
- 능동추론 에이전트의 선호(prior preference) 설계 문제(soft/hard/goal shaping) 자체가 별도 연구 주제 — [arXiv 2512.03293](https://arxiv.org/pdf/2512.03293)

### Inferences
- 실무 결론: **FEP/AIF를 "새 학습 알고리즘"으로 쓰지 말고 "설계 언어"로 써라.** 가져올 것은 (1) 지각·행동을 같은 목적함수(예측오차/놀라움 최소화)로 다루는 통합, (2) 선호를 보상이 아니라 "선호 관측 분포"(set-point)로 두는 관점(4·5번의 항상성과 직결), (3) 정보 이득(epistemic value)을 행동 선택에 더하는 관행. 이 셋은 표준 RL/모델기반 제어에 모듈로 추가 가능하다.
- AXIOM의 교훈은 "FEP가 이겼다"가 아니라 **"강한 객체중심 prior + 온라인 성장하는 혼합모델 + 주기적 모델 축소(수면 중 통합과 유사)"**가 10k 스텝 영역에서 DRL을 이긴다는 것이다. 이는 1번(선천 prior)과 4번(통합/가지치기)의 원리와 일치한다. 픽셀 일반 도메인·장기 신용할당으로의 확장은 미입증.
- 소형 장치에서는 이산 AIF(pymdp류)가 **의사결정 레이어**(수십 상태·수 개 행동의 메타 제어: 탐색 vs 활용, 어느 모듈을 켤지)로는 적합하나 지각 레이어로는 부적합하다.

### Gaps
- pymdp/JAX 버전의 2025~26년 실제 성능 벤치마크(연속 상태, 수백 차원)는 찾지 못했다.
- AXIOM 결과의 독립 재현 여부는 확인하지 못했다(보도자료는 "third-party validated"라 하나 출처 미확인).

---

## 4. 상보적 학습 시스템(CLS): 해마의 빠른 에피소드 학습 vs 신피질의 느린 의미 학습, 리플레이·수면 통합, AI 지속학습으로의 전이와 남은 실패

### Takeaway
CLS는 "빠르고 희소한(pattern-separated) 해마 저장 + 리플레이를 통한 느린 신피질 통합"으로 파국적 망각을 피한다는 확립된 틀이며 [확립/이론], 2016년 갱신판은 리플레이가 단순 반복이 아니라 **목표 가중(우선순위)·일반화·스키마 일치 시 빠른 신피질 학습**을 포함한다고 명시한다. AI에서는 experience replay가 가장 신뢰할 만한 지속학습 기법이지만 계산 비용·안정성-가소성 트레이드오프·"가소성 상실"·모델 용량 포화라는 네 가지 벽이 남아 있다 [AI 부분 성공]. 2026년에는 해마 없이 선조체 단독으로 절차기억 리플레이가 일어난다는 결과가 나와 "리플레이는 해마 전유물이 아닌 병렬 메커니즘"임이 드러났다.

### Cited Findings
- **CLS 2016 갱신판 핵심.** 두 학습 시스템: "The first gradually acquires structured knowledge representations while the second quickly learns the specifics of individual experiences"; 해마 리플레이가 목표에 따라 경험을 가중(weighting)하며, 해마 활동 패턴이 특정 일반화를 지원하고, 정보가 기존 구조와 일치하면 신피질 학습이 빠를 수 있음; 신경과학-AI 설계의 연결을 강조 — [Kumaran, Hassabis, McClelland 2016 TICS](https://doi.org/10.1016/j.tics.2016.05.004)
- **수면 중 시스템 통합 기전.** "repeated neuronal replay of representations originating from the hippocampus during slow-wave sleep"가 정보를 신피질로 점진 통합; 해마 리플레이, 수면 진동(느린 진동·방추·sharp-wave ripple)이 정보 흐름과 가소성을 조절하며, 기억은 "abstracted, gist-like representations"로 "qualitative transformations"를 겪음 — [Klinzing, Niethard, Born 2019 Nat Neurosci](https://www.nature.com/articles/s41593-019-0467-3); 큰 SWR이 해마-피질 재활성화·통합을 촉진 — [Neuron 2025](https://www.cell.com/neuron/abstract/S0896-6273(25)00756-1); 종합 리뷰 — [Sleep's contribution to memory formation, Physiol Rev 2024](https://journals.physiology.org/doi/pdf/10.1152/physrev.00054.2024); 각성 중 리플레이도 통합·계획에 기여 — [Awake replay, TINS 2025](https://www.cell.com/trends/neurosciences/fulltext/S0166-2236(25)00037-2)
- **리플레이의 우선순위.** 수면 중 특정 트랙의 리플레이 비율은 동물이 그 궤적을 달린 횟수에 비례해 증가(경험 빈도 가중) — [Nat Commun 2023](https://www.nature.com/articles/s41467-023-43939-z)
- **해마 독립 리플레이(2026).** "replay occurs in the dorsal striatum during offline consolidation of a procedural memory in mice, independently of the hippocampus, and that its content predicts subsequent performance improvements"; 현저한 행동 사건 관련 시퀀스가 우선 리플레이되며 정/부 결과가 개별 리플레이에 반대 효과; "All features of replay persisted despite complete bilateral hippocampal lesions" → "replay-driven memory consolidation can operate through parallel, independent mechanisms" — [Thompson et al. 2026 Nat Neurosci](https://www.nature.com/articles/s41593-026-02362-5)
- **왜 통합하는가(2026 이론).** 통합은 안정화가 아니라 일반화 최적화를 위한 "predictive forgetting" — "the selective retention of experienced information that predicts future outcomes"; 예측력 없는 정보는 가지치기되고 고용량 네트워크일수록 반복 정제 사이클의 이득이 큼 — [Why the Brain Consolidates, arXiv 2603.04688](https://arxiv.org/abs/2603.04688)
- **AI 지속학습 지형(van de Ven 2024).** 세 시나리오(task-/domain-/class-incremental) 중 class-incremental이 가장 어렵고 "inferring task identity ... is often found to be particularly difficult"; replay는 "especially for large and complex continual learning problems, replay seems to play an important and perhaps necessary role"이나 "potentially high computational cost"; EWC류 파라미터 정규화는 "often struggle to learn inter-task boundaries in class-incremental learning"; 생성적 리플레이는 "difficult to train generative models of decent quality, especially in an incremental setting"; 미해결: "reduced forgetting often comes at the cost of an impaired ability to learn new information", "substantial loss of plasticity", 자원 효율, 정제되지 않은 실데이터 — [van de Ven, Soures, Kudithipudi 2024 (ar5iv)](https://ar5iv.labs.arxiv.org/html/2403.05175)
- **EWC = 시냅스 통합의 모사.** "selectively slow[s] down learning on the weights important for those tasks"로 MNIST·순차 Atari에서 망각 완화 — [Kirkpatrick et al. 2017 PNAS](https://doi.org/10.1073/pnas.1611835114)
- **가소성 상실.** 표준 딥러닝은 지속학습에서 "progressively deteriorating until performing no better than shallow networks"; "plasticity is maintained indefinitely only by algorithms that continually inject diversity"; 해법 continual backpropagation은 저활용 유닛의 소수를 무작위 재초기화 — "a random, non-gradient component to maintain variability and plasticity"가 필요 — [Dohare et al. 2024 Nature](https://www.nature.com/articles/s41586-024-07711-7)
- **LLM에서의 한계.** 리플레이는 망각을 줄이지만 "there is a lower bound on forgetting arising from limited model capacity — a model pretrained close to saturation might not have sufficient capacity to absorb new information without overwriting" — [Forgetting in Language Models, arXiv 2605.26097](https://arxiv.org/pdf/2605.26097); 리플레이 예산을 키우면 안정성은 오르나 가소성은 떨어지는 트레이드오프 — [Scalable Strategies for CL with Replay, arXiv 2505.12512](https://arxiv.org/pdf/2505.12512); 서베이 — [CL for LLMs survey](https://arxiv.org/pdf/2402.01364)
- 리플레이/리허설 best practice 분석 — [Krawczyk & Gepperth 2024](https://www.researchgate.net/publication/384415203_An_analysis_of_best-practice_strategies_for_replay_and_rehearsal_in_continual_learning); pattern separation/completion을 명시 모델링한 CLS 신경망(2025) — [arXiv 2507.11393](https://arxiv.org/abs/2507.11393); 뇌 기능에서 영감받은 CL 종합 서베이 — [Aslam 2025, IJIS](https://onlinelibrary.wiley.com/doi/10.1155/int/3145236)

### Inferences
- 설계 원리 4 — **두 저장소 + 오프라인 통합 루프.** (a) 빠른 저장소: 비모수(k-NN/벡터 DB)·희소 코드·pattern separation(해시/랜덤 투영)으로 1회 노출 저장; (b) 느린 저장소: 파라미터 모델; (c) "수면" 단계에서 우선순위 리플레이(예측오차·보상·빈도·현저성 가중)로 (a)→(b) 증류. Thompson 2026은 절차 모듈(정책/스킬)에도 **자체 리플레이 버퍼**를 두라는 근거다: 리플레이는 중앙 해마 하나가 아니라 모듈별 병렬 메커니즘이어야 한다.
- 설계 원리 5 — **통합 = 예측적 망각.** 버퍼에서 무엇을 증류할지 기준은 "미래 결과를 예측하는가"이지 "최근인가"가 아니다(2603.04688). 구현: 샘플별 outcome-예측 기여도를 추정해 가중, 낮은 것은 버린다.
- 설계 원리 6 — **가소성 예산 관리.** Dohare 2024에 따라 저활용 유닛의 주기적 재초기화(비기울기 다양성 주입)를 통합 단계에 넣고, LLM 결과대로 모델 용량이 포화하면 리플레이로도 못 막으므로 **용량 여유(headroom)와 모듈 추가(성장)**를 설계에 포함하라.
- 실패 경고: 생성적 리플레이는 생성기가 약하면 역효과; class-incremental 설정에서 정규화만(EWC)으로는 부족; 리플레이 비용은 소형 장치에서 실제 병목이다(저해상도 임베딩 리플레이로 완화).

### Gaps
- Kumaran 2016 본문(스키마 일치 시 신피질 급속 학습의 구체 조건, DQN replay와의 연결 서술)은 접근 차단(403)으로 초록만 확인했다.
- 수면 통합에서 "어느 정도 비율의 경험이 통합되는가" 같은 정량 수치는 확보하지 못했다.

---

## 5. 신경조절물질 = 학습 신호: 도파민 RPE, ACh/NE의 불확실성 신호, 세로토닌, 삼인자 규칙, 그리고 가치의 원천으로서의 항상성/내수용·알로스타시스

### Takeaway
도파민 위상 반응이 보상예측오차(RPE)를 닮는다는 것은 확립 사실이나 [확립], 그것이 *유일한* 기능인지는 2022~2025년에 강하게 도전받았다(ANCCR 인과학습, 분포 RL, 램프 현상) [논쟁]. ACh=기대된 불확실성, NE=예상 못한 불확실성(Yu & Dayan), 세로토닌=불확실성에 따른 학습률 메타조절(Grossman 2022)은 "학습률·이득을 전역 스칼라로 조절하는 신호"라는 공통 구조를 가진다. 이들은 삼인자 규칙(pre × post × modulator, 적격성 흔적)으로 시냅스 가소성을 게이팅한다 [확립]. 가치의 원천은 외부 보상이 아니라 신체 항상성/알로스타시스의 예측오차라는 이론(Barrett & Simmons EPIC, 2025 Neuron)이 유력하며 AI 구현은 아직 개념 단계다.

### Cited Findings
- **도파민 RPE.** 영장류 도파민 뉴런의 출력 변동이 "signals changes or errors in the predictions of future salient and rewarding events"이며 적응 최적화 제어(TD) 이론으로 이해됨 — [Schultz, Dayan, Montague 1997 Science](https://doi.org/10.1126/science.275.5306.1593)
- **분포 코드.** 뇌가 미래 보상을 "not as a single mean, but instead as a probability distribution"으로 표현하며 생쥐 VTA 단일유닛이 이를 지지 — [Dabney et al. 2020 Nature](https://www.nature.com/articles/s41586-019-1924-6)
- **대안: 인과학습(ANCCR).** 도파민이 "Adjusted Net Contingency for Causal Relations"를 표상해 의미 있는 사건 후 **역방향으로** 원인을 찾는 학습 신호라는 가설; 도파민 램프는 "fundamentally inconsistent with the original conception of TDRL RPE"; RPE 모델군은 버전마다 기전이 달라 "family"이며 반증 가능성 우려 — [Jeong et al. 2022 Science](https://www.science.org/doi/10.1126/science.abq6740); [The Transmitter: Dopamine and the need for alternative theories](https://www.thetransmitter.org/dopamine/dopamine-and-the-need-for-alternative-theories/); 리뷰 — ["But why?" Dopamine and causal learning, Curr Opin Behav Sci 2024](https://www.sciencedirect.com/science/article/abs/pii/S2352154624000949); Gershman의 "prediction errors and beyond" 정리 — [Gershman 2024](https://gershmanlab.com/pubs/Gershman24_dopamine.pdf)
- **RPE 지지(2024).** 쥐 시간 도박 과제에서 NAcc 도파민은 "scaled monotonically with offered reward volume"하고 블록 기대에 반비례(저블록 20μL 상승, 고블록 하락)하여 RPE 특성; 그러나 행동은 블록 초반에 높은 학습률을 쓰는데 도파민은 이를 반영하지 않아 "dopamine-independent mechanisms instantiate dynamic learning rates" — [Cell Reports 2024 (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11571066/)
- **ACh/NE = 불확실성.** "Acetylcholine signals expected uncertainty, coming from known unreliability of predictive cues within a context. Norepinephrine signals unexpected uncertainty, as when unsignaled context switches produce strongly unexpected observations"; 두 신호는 "part-antagonistic, part-synergistic" — [Yu & Dayan 2005 Neuron](https://doi.org/10.1016/j.neuron.2005.04.026); LC-NE가 지각 민감도를 선택적으로 높여 공간주의에 기여 — [PMC 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC11223979/)
- **세로토닌 = 학습률 메타학습.** 배측 봉선핵 세로토닌 뉴런이 기대/예상 못한 불확실성 둘 다를 추적하고, 억제 시 "simulated meta-learning lesion" 예측과 일치하는 학습 변화 → 불확실성이 학습 속도를 조절("slowly in stable environments but faster during environmental changes") — [Grossman, Bari, Cohen 2022 Curr Biol](https://doi.org/10.1016/j.cub.2021.12.006); 세로토닌 자극이 미래 보상에 대한 인내를 높이되 보상 확률·타이밍 불확실성에 따라 효과가 바뀜 — [Miyazaki et al. 2018 Nat Commun](https://www.nature.com/articles/s41467-018-04840-2); 세로토닌·도파민의 모델기반 학습 상보성 리뷰 — [Curr Opin Behav Sci 2025](https://www.sciencedirect.com/science/article/pii/S2352154624001153); 세로토닌의 가치 예측 코딩 주장도 공존(논쟁) — [bioRxiv 2023](https://www.biorxiv.org/content/10.1101/2023.09.19.558526v1.full)
- **삼인자 규칙과 적격성 흔적.** 가소성이 pre·post 활동 + 신경조절 "third factor"(도파민·ACh·NE·세로토닌의 위상 신호)를 요구하며, 시냅스에 남는 eligibility trace가 행동 시간척도(초)에서 뒤늦은 조절 신호와 결합 — [Gerstner et al. 2018 Front Neural Circuits](https://www.frontiersin.org/journals/neural-circuits/articles/10.3389/fncir.2018.00053/xml/nlm); [arXiv 1801.05219](https://arxiv.org/pdf/1801.05219); 선조체 모델에서 가소성이 "cholinergic pauses that temporally align with behaviorally relevant events"로 게이팅 — [PMC 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12504730/)
- **간접경로의 삼인자 규칙.** 간접경로 활동은 선조체 도파민이 **억제될 때** 강화되어 나쁜 결과를 낳은 행동 회피를 학습하고, 그 활성화는 새 행동을 개시해 탐색을 허용 — [Lee & Sabatini 2025 Nat Rev Neurosci](https://www.nature.com/articles/s41583-025-00925-2)
- **가치의 원천 = 알로스타시스/내수용.** EPIC 모델: 무과립(agranular) 내장운동 피질(전측 뇌섬엽·대상·내측 전전두)이 "send visceromotor predictions to the body and transmit interoceptive predictions about the viscerosensory consequences"; 이들은 "issue allostatic visceromotor predictions to the hypothalamus, brainstem and spinal cord nuclei to maintain homeostasis"; 층 구조 논증: 무과립 피질은 L4가 없고 심층 투사뉴런이 많아 예측 송신에, 과립 감각피질은 L2/3 투사뉴런이 많아 오차 계산에 적합; 정동(affect)은 "simulated interoceptive sensations (that is, from interoceptive predictions)"에서 유래하고 무과립 영역은 "relatively insensitive to prediction-error signals" — [Barrett & Simmons 2015 Nat Rev Neurosci (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4731102/); 2025 Neuron은 분산 알로스타시스 시스템이 전뇌 신호를 조직하고 "places bodily regulation at the core of brain structure"라고 주장(검색 요약) — [It's not the thought that counts: Allostasis at the core of brain function, Neuron 2025](https://www.cell.com/neuron/fulltext/S0896-6273(25)00716-0)
- **AI로의 이식 시도 [초기 단계].** 내수용 원리를 AI 규제 구조로 옮기는 개념 프레임(homeostatic/allostatic/enactive 세 원리, 내부 상태 변수와 규제 루프) — 구현 결과 없음 — [Candia-Rivera 2026, Interoceptive machine framework](https://arxiv.org/abs/2604.24527); NE 버스트의 일시적 이득 상승을 모사한 "dynamic gain scaling"이 MNIST/CIFAR/mini-ImageNet의 domain/class-incremental에서 "effectively attenuates stability gaps while maintaining competitive accuracy" — [arXiv 2507.14056](https://arxiv.org/abs/2507.14056); DA/ACh/5-HT/NA의 다중 조절 동역학을 ANN에 넣자는 "conceptual study"(Go/No-Go 과제 수준) — [arXiv 2501.06762](https://arxiv.org/abs/2501.06762); end-to-end 신경조절 제어 제안 — [NeuMoSync 2026](https://arxiv.org/pdf/2608.04358)

### Inferences
- 설계 원리 7 — **학습 신호를 "조절 벡터"로 분리하라.** 하나의 스칼라 보상 대신 최소 4채널: (i) RPE류(가치·정책 갱신 게이트, 분포형으로 두면 위험 감수 정책에 유리), (ii) 기대 불확실성(알려진 잡음 → 해당 채널 학습률·이득 ↓, 주의 재배분), (iii) 예상 못한 불확실성(컨텍스트 전환 → 전역 학습률·탐색 일시 ↑, "리셋"), (iv) 메타 학습률(환경 변동성 추정에 따른 학습률 스케줄). 각 채널은 삼인자 규칙처럼 **적격성 흔적 × 조절 신호**로 모듈 가중치 갱신을 게이팅한다.
- 설계 원리 8 — **회피 학습을 별도 경로로.** Lee & Sabatini의 간접경로 규칙(도파민 억제 시 강화)은 "나쁜 행동을 지우는" 것이 아니라 "회피+대안 개시"를 학습하는 별도 가중치 집합을 뜻한다. Go/NoGo 두 헤드를 두고 음의 RPE가 NoGo 헤드를 강화하도록 하면 탐색이 자연히 생긴다.
- 설계 원리 9 — **가치는 내부 상태 변수의 예측오차에서 도출.** 에이전트에 에너지/연산 예산·오류율·데이터 신선도·사용자 만족 같은 내부 변수와 set-point를 두고, 보상 = "예측된 편차의 감소"(알로스타시스)로 정의하면 외부 보상 설계 없이도 동기 신호가 생긴다. 다만 이것은 2026년 현재 **개념 수준**이며 검증된 벤치마크가 없다.
- 도파민 논쟁의 교훈: RPE는 "표준 과제에서 잘 맞는 1차 근사"다. 인과 구조 학습(역방향 원인 탐색)과 분포 표현을 보조로 두되, 단일 TD 오차에 모든 것을 걸지 말라.

### Gaps
- 2025 Neuron 알로스타시스 논문은 접근 차단으로 검색 요약만 확인했다.
- 신경조절물질 영감 AI 기법이 **대규모 벤치마크에서 표준 CL 기법을 넘었다는 증거는 찾지 못했다**(소형 데이터셋 결과만 존재).
- 노르아드레날린의 "네트워크 리셋" 가설(Bouret & Sara)과 LC 위상/긴장 모드(Aston-Jones & Cohen)의 1차 문헌은 수집하지 못했다.

---

## 6. 기저핵 행동선택, 소뇌 전방모델, 전전두 작업기억과 "전역 작업공간" 방송: 용량 한계의 모습

### Takeaway
기저핵은 피질 후보 행동 중 하나를 탈억제로 선택하고(직접/간접 경로, 도파민 삼인자 학습) [확립, 세부는 논쟁], 소뇌는 교사신호(등반섬유)로 운동 명령의 감각 결과를 ~100ms 앞서 예측하는 전방모델을 학습하며 [확립], 전전두-두정 "전역 작업공간"은 200~300ms의 비선형 점화(ignition)로 한 번에 **하나의** 내용을 전역 방송한다 [이론, 증거 다수]. 용량 한계는 작업기억 ~4 청크, 의식적 접근의 직렬 병목(attentional blink, PRP)으로 나타난다.

### Cited Findings
- **기저핵 모델들.** 전통 Go/No-Go(직접=촉진, 간접=억제) 외에 center(직접)-surround(간접)-context(간접)의 "Triple-control" 모델이 기존 모델로 설명 안 되는 관찰을 재현 — [PMC 2023](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10522336/); 반면 선조체 자극이 "context-specific reinforcement and not action selection"을 지지한다는 2025년 결과도 있어 행동선택 모델이 선조체 기능을 다 담지 못한다는 반론 존재 — [Cell Reports 2025](https://www.cell.com/cell-reports/fulltext/S2211-1247(25)00822-8); 간접경로의 회피·새 행동 개시 역할 — [Lee & Sabatini 2025](https://www.nature.com/articles/s41583-025-00925-2)
- **기저핵 vs 소뇌 분업.** 피질-기저핵 루프는 "novelty-based motor prediction error"로 원하는 결과에 맞는 구체 행동을 학습하고, 소뇌는 "minimizes the remaining aiming error" — [PLoS Comput Biol 2023](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1011024); 모델기반 RL 행동선택에 전전두-기저핵·소뇌 네트워크가 관여 — [Sci Rep 2016 (PMC)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4990901/)
- **소뇌 전방모델.** 감각 피드백 지연(시각 30~80ms, 신경전도 10~100ms)을 보상하기 위해 "predicts the state of the body time by time"; 세 역할: 상태 예측, 예측오차 계산(운동학습), 최적 피드백/칼만 이득 계산. 치상핵 출력이 미래 이끼섬유 입력을 예측("DNC firing rates at time t contained predictive information about the future input ... at time t+t₁"); 건강인의 손목 추적 F1 성분 지연 ~60ms(피드백으로 설명 불가), 소뇌 환자는 ~100ms로 지연·불안정; 회로는 "superb homogeneity ... 'crystal-like'"이며 출력(~0.8M 섬유)은 피질 입력(~21M)의 5% 미만으로 압축; 예측 시간 100~200ms; 인지 기능으로의 확장은 "speculative" — [Tanaka et al., The Cerebro-Cerebellum as a Locus of Forward Model (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7160920/)
- **전역 신경 작업공간(GNW).** 점화 = "the sudden, coherent, and exclusive activation of a subset of workspace neurons coding for the current conscious content", 자극 후 "around 200 to 300 ms"에 전전두·두정으로 비선형 전파; 약한 놓친 자극은 "transient, decaying PFC activity"만 남김(van Vugt 2018); 하드웨어는 "large pyramidal neurons with long-distance axons"의 전전두-두정 상호연결, "the GNW is not a localizationist theory"; 지속 활동 루프는 "loop through subcortical regions with an important role for the thalamus and cerebellar nuclei"(중앙 시상 자극으로 영장류 마취 역전); **병목**: 한 순간 하나의 표상만 전역 방송 → attentional blink, psychological refractory period, 이중과제 시 접근 지연("delayed by much more than 300 ms"); 작업기억과의 관계: "the attended working memory item is conscious and uses the GNW for broadcasting", 주의 밖 항목은 "weaker persistent firing within local processors or by activity-silent synaptic mechanisms"; 미해결: 현상적 vs 접근 의식, 발달 시점(5개월 영아 900ms vs 성인 300ms), 자기의식 — [Mashour, Roelfsema, Changeux, Dehaene 2020 Neuron (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8770991/); 척추동물에서의 진화적 기원 — [Neurosci Conscious 2023](https://academic.oup.com/nc/article/2023/1/niad020/7272926)
- **작업기억 용량.** 정상 성인의 주의 초점 용량은 "about four chunks" — [Cowan 2001](https://www.researchgate.net/publication/11830840_The_magical_number_4_in_short-term_memory_A_reconsideration_of_mental_storage_capacity); 지속 발화 모델 vs 단기 시냅스 가소성에 기반한 "activity-silent" 모델의 논쟁 — [PMC 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10158524/); 시냅스 이론으로 청킹을 설명(4개 클러스터 활성 시 청킹 클러스터가 이전 자극 클러스터를 억제) — [Synaptic Theory of Chunking in WM](https://arxiv.org/pdf/2408.07637)
- GNW 아이디어를 AI 일반지능 설계에 연결한 논의 — [Juliani et al., arXiv 2204.05133](https://arxiv.org/pdf/2204.05133)

### Inferences
- 설계 원리 10 — **행동 선택은 탈억제형 WTA + 두 헤드(Go/NoGo) + 조절 학습.** 모든 모듈이 후보 행동을 "제안"하고 중앙 선택기가 하나만 통과시키되, 선택기 가중치는 RPE류 신호(양→Go 강화, 음→NoGo 강화)로 갱신한다. Cell Reports 2025의 반론은 "선택기 = 순수 라우터"가 아니라 **맥락 의존 강화 저장소**임을 시사하므로 상태 임베딩을 선택기 입력에 포함하라.
- 설계 원리 11 — **모든 효과기에 작은 전방모델.** 소뇌 원리: 각 행동(도구 호출·API·모터)마다 "명령→결과" 예측기를 **지도학습**(실제 결과 = 교사)으로 두고, 결과 도착 전 예측으로 다음 단계를 진행하며, 예측오차를 (a) 전방모델 갱신, (b) 상위 모듈의 놀라움 신호로 쓴다. 균일한 작은 모듈을 다수 복제하는 "crystal-like" 구조가 적합하다.
- 설계 원리 12 — **전역 작업공간은 직렬 병목이 핵심 기능.** 단일 슬롯(또는 ~4 청크)의 전역 버퍼에 점화 임계값(비선형 경쟁)을 두고, 통과한 내용만 모든 모듈에 방송한다. 병렬 모듈의 대부분은 전역 버퍼를 거치지 않는다(무의식 처리). 이것은 비용을 절약하면서 모듈 간 조정을 가능하게 하는 뇌의 해법이며, LLM의 "모든 토큰이 모든 토큰을 본다"와 정반대다.
- 활동-침묵 작업기억(단기 시냅스 가소성)은 **"상태를 가중치에 잠깐 저장"**하는 저전력 트릭이다: 지속 발화(=계속 연산) 대신 빠른 감쇠 가중치(fast weights)로 컨텍스트를 들고 다니면 에너지가 절약된다.

### Gaps
- 기저핵 직접/간접 경로의 정확한 계산 역할은 2025년 현재도 모델 간 불일치가 크다(Go/No-Go vs Triple-control vs 맥락 강화). 어느 쪽이 맞는지는 이 노트로 판정 불가.
- 전전두 작업기억의 신경 코드(지속 발화 vs activity-silent)의 결론은 미확정.

---

## 7. 국소 학습규칙 vs backprop: Hebbian/STDP, 삼인자, PC≈backprop, forward-forward — 2026년 기준 유용한 규모에서 작동하는가

### Takeaway
2026년 기준으로 **"backprop 없는 국소 규칙이 ImageNet급에서 backprop과 동등"하다는 결과는 없다** [AI 미검증]. 가장 근접한 것은 (a) sign-symmetry/feedback alignment류(ImageNet에서 backprop에 근접), (b) 재귀 스파이킹망의 e-prop(BPTT에 근접), (c) 블록 내부는 backprop을 쓰는 하이브리드 FF이다. 순수 Hebbian(SoftHebb)은 ImageNet 27.3%, FF-CNN은 CIFAR-100 ~37%에 머문다. 즉 소비자 PC용 에이전트는 backprop(또는 블록-국소 backprop)을 기본으로 쓰고, 국소 규칙은 저장 활성이 없는 온라인 적응·스파이킹 하드웨어 같은 특수 상황에만 쓰는 것이 현실적이다.

### Cited Findings
- **Forward-Forward.** backprop의 전·후방 패스를 "two forward passes, one with positive (i.e. real) data and the other with negative data"로 대체, 층마다 goodness(제곱 활동 합)를 양성에서 높이고 음성에서 낮춤; 양·음 패스를 시간상 분리하면 "the negative passes could be done offline"; 논문 스스로 "preliminary investigations", "a few small problems" — [Hinton 2022](https://arxiv.org/abs/2212.13345)
- **FF 규모 확장 시도.** 층별 독립 훈련을 MobileNetV3·ResNet18로 확장해 "performance comparable to standard backpropagation" 주장; 블록 내부만 backprop을 쓰는 하이브리드가 "tends to outperform backpropagation baselines"; ImageNet 성능·실패 사례 언급 없음 — [Scalable Forward-Forward 2025](https://arxiv.org/abs/2501.03176); FF로 훈련한 CNN은 CIFAR-100 약 37% — [Sci Rep 2025](https://www.nature.com/articles/s41598-025-26235-2); Hebbian FF(기울기 없음)는 MNIST/FashionMNIST에서 FF와 같은 정확도·더 빠른 훈련 — [HebbFF 2025](https://opt-ml.org/papers/2025/paper147.pdf)
- **순수 Hebbian(SoftHebb).** 피드백·타깃·오차 신호 없이 최대 5 은닉층: MNIST 99.4%, CIFAR-10 80.3%, STL-10 76.2%, **ImageNet 27.3%** — [Journé et al., Hebbian Deep Learning Without Feedback](https://arxiv.org/abs/2209.11883)
- **ImageNet에서 통한 것.** 여러 생물학적 타당 알고리즘이 MNIST/CIFAR에서는 되지만, ImageNet에서는 sign-symmetry만 "classification performance approaching that of backpropagation-trained networks" — [Xiao et al. 2018](https://arxiv.org/abs/1811.03567); 2025년 스파이크 기반 양방향 증류(BSD)가 MLP/CNN/RNN/AE에서 backprop과 비슷하다고 주장 — [arXiv 2509.20284](https://arxiv.org/abs/2509.20284); 연속시간 모델에서 SGD/FA/DFA/Kolen-Pollack이 극한으로 도출 — [arXiv 2510.18808](https://arxiv.org/html/2510.18808v1)
- **삼인자 SNN 학습 개관(2025).** Δw = H(t_pre, t_post, g_t); e-prop은 "close to BPTT on speech recognition, word prediction, one-shot learning, and pattern generation", MDGL은 "closely matches BPTT and outperforms eprop in online learning", ETLP는 N-MNIST/SHD에서 경쟁력(수치 미제시); 한계: "Simplified neuron models", "rely on simulations and synthetic datasets", "careful parameter tuning", "Limited scalability; not tested on large-scale networks", "Lack of clear mechanism for global error propagation in complex tasks"; 신경모방 칩 구현은 "Relative scarcity" — [Three-factor learning in SNNs overview, arXiv 2504.05341](https://arxiv.org/html/2504.05341v2); e-prop 원 논문: "approaches the performance of backpropagation through time (BPTT)" — [Bellec et al. 2020 Nat Commun](https://doi.org/10.1038/s41467-020-17236-y)
- **PC 네트워크.** CIFAR-10 수준, 대규모 데이터셋에서 성능 급락, 불안정·느림 — [PCN intro 2025](https://arxiv.org/pdf/2506.06332); PCX: 확장성이 "major open problem" — [PCX](https://arxiv.org/abs/2407.01163); prospective configuration은 효율·생물학적 일치를 주장하나 소형 과제 — [Song et al. 2024](https://www.nature.com/articles/s41593-023-01514-1)
- **왜 뇌는 backprop을 못 쓰나(그리고 근사 후보).** Richards et al.은 feedback alignment("random synaptic feedback weights"), predictive coding(국소 Hebbian), 수상돌기(apical dendrite) 기전을 기울기 근사 후보로 제시 — [Richards et al. 2019 (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7115933/); Millidge 리뷰의 가중치 대칭·도함수 전달·E/I 균형 문제 — [Millidge et al.](https://ar5iv.labs.arxiv.org/html/2107.12979)

### Inferences
- 설계 원리 13 — **"국소성"은 알고리즘이 아니라 아키텍처로 얻어라.** 뇌의 국소성은 모듈이 작고 루프가 짧기 때문이다. 소프트웨어에서는 **모듈 단위 backprop(블록-국소 손실) + 모듈 간 통신은 저차원 예측/오차/조절 신호**로 두면 "전역 backprop 없음"의 실용적 이득(메모리·병렬성·모듈 교체 가능성)을 챙기면서 성능 손실을 피한다(Scalable FF의 하이브리드 결과와 일치).
- 온라인·스트리밍 적응(저장 활성 없음, 단일 패스)에는 e-prop형 적격성 흔적 + 삼인자 규칙이 실제 선택지다. 특히 재귀 모듈·시계열 센서에 적합.
- Hebbian 단독은 전처리/특징 추출층(비지도, 데이터 분포 추적)에만 쓰고 결정층은 기울기 기반으로 두는 것이 2026년의 합리적 분업이다.

### Gaps
- Scalable FF(2501.03176)·BSD(2509.20284)의 "backprop과 동등" 주장은 초록 기반이며 ImageNet 수치·독립 재현 여부를 확인하지 못했다.
- 2025~26년에 PC/FF/Hebbian이 ImageNet-1k에서 backprop 대비 어느 수준인지의 **단일 표**는 찾지 못했다.

---

## 8. 에너지: 20W 예산이 함의하는 것(희소 발화, 이벤트 구동)과 신경모방 하드웨어(Loihi 2/Hala Point, SpiNNaker2, BrainChip)의 2026년 현황

### Takeaway
20W 예산의 핵심 함의는 두 가지다: (1) 스파이크가 비싸므로 **동시 활성 뉴런 ≤1~15%**(Lennie: ~2%)의 희소 코드가 강제되고, (2) 인간 피질에서 **통신(장거리 전달)이 계산보다 35배 비싸다**(3.5W vs 0.1W) — 즉 뇌의 효율은 "연산"보다 "데이터 이동 최소화·국소성"에서 온다 [확립]. 신경모방 하드웨어는 희소·이벤트 구동·저지연 워크로드(센서, 키워드, 최적화)에서 100~1000배 에너지 이득을 **측정으로** 보이지만, 밀집 워크로드에서는 GPU에 못 미치고 소프트웨어 생태계가 "빠진 층"이라 범용 이득은 아직 아니다 [부분 성공]. 상용 성숙 제품은 BrainChip Akida 정도이고 Hala Point는 "research prototype"이다.

### Cited Findings
- **에너지 예산.** 설치류 회백질에서 "Action potentials and postsynaptic effects of glutamate are predicted to consume much of the energy (47% and 34%, respectively)", 휴지전위 13%, 글루타메이트 재활용 3%; 에너지 사용은 발화율에 강하게 의존 → "distributed codes, with ≤15% of neurons simultaneously active" 예측 — [Attwell & Laughlin 2001 J Cereb Blood Flow Metab](https://doi.org/10.1097/00004647-200110000-00001)
- Lennie: 스파이크 비용이 높아 "possibly to fewer than 1%" 또는 "1/50th of the population of cortical neurons could afford to be significantly active"; 스파이크당 2.4×10⁹ ATP, 그중 52%가 후시냅스 EPSC; 영역 간 유연한 에너지 배분 필요(검색 요약) — [Lennie 2003 Curr Biol](https://www.cell.com/current-biology/fulltext/S0960-9822(03)00135-0)
- **통신 > 계산.** 인간 피질 에너지 감사: "communication costs are 35-fold computational costs" — 계산에 0.1W ATP, 장거리 통신에 3.5W; 뉴런의 계산 비용은 이론적 최적 bits/joule 대비 "off by a factor of 10⁸"; 최적화 모델은 뉴런 발화에 약 2,000~2,500 시냅스 입력을 예측 — [Levy & Calvert 2021 PNAS](https://www.pnas.org/doi/10.1073/pnas.2008173118)
- **Hala Point(Intel, 2024).** 1,152개 Loihi 2(Intel 4 공정), 140,544 코어, 1.15B 뉴런·128B 시냅스, 최대 2,600W, "up to 20 petaops", 기존 DNN에서 "15 trillion 8-bit operations per second per watt"(MLP, 10:1 희소·10% 활성 조건), 최적화 과제에서 CPU/GPU 대비 "100 times less energy"·"50 times faster"; Sandia 배치; "Currently, Hala Point is a research prototype" — [Intel newsroom](https://www.intel.com/content/www/us/en/newsroom/news/intel-builds-worlds-largest-neuromorphic-system.html)
- **규모 로드맵(Nature 2025).** 신경모방 설계는 "often for applications with size, weight and power constraints"; 확장 아키텍처·응용·과제·생태계를 정리 — [Kudithipudi et al. 2025 Nature](https://www.nature.com/articles/s41586-024-08253-8). (검색 요약에 따르면 같은 리뷰가 2026년까지 100억 뉴런급 시스템 등장과 "gradient-based training of SNNs ... off-the-shelf", 아날로그→디지털 전환을 언급하나 본문 확인은 못 했음)
- **상용화 평가(2025).** "Gradient-based training methods for deep spiking neural networks have become readily available", 디지털 설계가 아날로그/혼성을 대체, 인메모리 컴퓨팅이 시장 근접; 남은 두 과제는 "developing programming frameworks for general-purpose neuromorphic systems and scaling production deployment"; 적합 응용은 배터리 구동·IoT 엣지·개인 전자기기 — [Muir & Sheik 2025 Nat Commun](https://www.nature.com/articles/s41467-025-57352-1)
- **플랫폼 비교 측정.** Loihi는 단순 벡터-행렬 곱에서, MAC 배열을 가진 SpiNNaker2 프로토타입은 고차원 벡터-행렬 곱에서 더 효율적(키워드 스포팅·적응 제어) — [Yan et al. 2021](https://iopscience.iop.org/article/10.1088/2634-4386/abf150/pdf); 인지 무선 과제에서 Loihi 2·TrueNorth·SpiNNaker·SpiNNaker2·Hala Point 벤치마크로 서브-ms 결정, 기존 대비 에너지 100~1000배 절감(검색 요약) — [Springer 2026](https://link.springer.com/article/10.1007/s44163-026-01093-7)
- **2026년 지형(2차 자료, 주의).** Loihi 2 칩당 1M 뉴런·120M 시냅스; TrueNorth 1M 뉴런 65~70mW; NorthPole은 ResNet-50에서 A100 대비 22배 에너지 효율(온칩 메모리로 대역폭 병목 제거); BrainChip Akida AKD1500 800 GOPS @ 300mW, Pico ≤1mW, "Only mature product with distribution and real customers"; SpiNNaker2 칩당 152 ARM Cortex-M4F, 1,000만 코어 목표; 결론: "The missing layer across all of them is software", 밀집 동기 워크로드에서는 GPU에 열세, 상용 견인은 Akida뿐 — [Wagenbach, Neuromorphic Hardware Landscape 2026 (블로그)](https://www.joshwagenbach.com/blog/neuromorphic-hardware-landscape-2026); 같은 취지의 2차 기사 제목 "22x More Efficient Than GPUs — and Still Far From the Market" — [PrezenceAI 2026](https://prezenceai.com/en/research/computacao-neuromorfita-loihi-northpole-eficiencia-inferencia-2026.html)
- Nature 2025 리뷰 관련 검색 요약: BrainChip 2세대 Akida는 1.2M 뉴런·100억 시냅스로 확장, AKD1000은 2022년부터 양산, 2025년 Akida Cloud 출시(검색 요약, 본문 미확인) — [Kudithipudi et al. 2025](https://www.nature.com/articles/s41586-024-08253-8)

### Inferences
- 설계 원리 14 — **소프트웨어에서 20W 원리를 구현하는 법은 "스파이크"가 아니라 "희소성 + 국소성 + 이벤트 구동"이다.** (a) 활성 희소성(MoE/top-k/조건부 실행: 한 번에 1~10% 모듈만 가동), (b) 통신 최소화(모듈 간에는 저차원 예측·오차·조절 벡터만 교환; Levy & Calvert의 35배 교훈), (c) 변화가 있을 때만 계산(기대 억제·델타 인코딩·웨이크업 트리거). 이 셋은 CPU/GPU에서 그대로 전력 절감으로 이어진다.
- 신경모방 칩은 2026년에 **"이벤트 센서 + 배터리 + 지연 민감"** 조합에서만 확실한 이득이 있다. 소비자 PC 에이전트에는 불필요하고, 소형 장치에서는 Akida급 상용 칩을 전처리(키워드/이상 탐지)에만 고려하라. 학습 알고리즘·툴체인은 아직 GPU 생태계에 비할 바가 아니다.
- 뇌의 "계산 효율"은 과장되어 있다(Levy & Calvert: 이론 최적 대비 10⁸배 비효율). 뇌가 이기는 지점은 연산당 에너지가 아니라 **무엇을 계산하지 않을지**(희소 코드·예측에 의한 억제·수면 중 오프라인 처리)를 결정하는 조직 원리다.

### Gaps
- Loihi 2·SpiNNaker2·Akida의 **2026년 공식 벤치마크 수치(동일 워크로드 GPU 대비)**는 1차 자료로 확보하지 못했고, 2차 블로그/기사에 의존했다(수치 신뢰도 중간).
- SpiNNcloud 상용 배치 규모, Loihi 3 로드맵은 확인하지 못했다(블로그의 "Loihi 3 2026 상용" 주장은 미검증).

---

## 9. 발달 신경과학: 시냅스 과잉생성·가지치기, 수초화 순서(감각→연합→전전두), 임계기와 분자 브레이크, 피질이 후방→전방으로 발달하는 이유

### Takeaway
피질은 **과잉생성 후 가지치기**(시각피질 생후 ~4개월 피크, 전전두 ~1년 피크, DLPFC 가시 과잉은 30대까지 지속)와 **후방→전방 수초화·성숙**(Gogtay: 고차 연합피질은 1차 감각·시각피질 이후에 성숙)을 따르며 [확립], 임계기는 PV 억제 뉴런 성숙으로 열리고 perineuronal net·수초 관련 억제 신호(Nogo 수용체, PirB)·Lynx1(콜린성 브레이크) 같은 **분자 브레이크**로 닫힌다 [확립, 인간 적용은 진행 중]. 이는 "하위 표상이 안정된 뒤 상위가 학습한다"는 위계적 커리큘럼과 "고가소성 창 → 통합·잠금"의 스케줄을 뜻한다.

### Cited Findings
- **후방→전방 성숙.** 4~21세 13명을 2년마다 MRI로 추적한 4D 지도에서 "higher-order association cortices mature only after lower-order somatosensory and visual cortices", 진화적으로 오래된 영역이 먼저 성숙 — [Gogtay et al. 2004 PNAS](https://www.pnas.org/doi/10.1073/pnas.0402680101)
- **과잉생성·가지치기의 시간표.** 시냅스 과잉생성 피크: 시각피질 생후 약 4개월 → 청각피질·각회·브로카 영역 → 내측 전전두 약 1세; 영역별 회백질 성숙 차이는 "heterochronous synaptic pruning"에서 비롯; 피질 수초화는 "begins posterior and moves anterior", 두정·전두엽이 마지막; DLPFC는 가장 늦게 성숙하며 가지치기와 수초화가 병행; 인간 DLPFC 피라미드 뉴런의 가시 과잉생성은 "up to the third decade of life" — [Normal Development of Brain Circuits (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3055433/); 가시 형성·가지치기 종설 — [PubMed 2023](https://pubmed.ncbi.nlm.nih.gov/37962796/); 청소년기 집행기능 발달과 시냅스 가소성 — [Transl Psychiatry 2013](https://www.nature.com/articles/tp20137)
- **분자 브레이크.** Lynx1 발현 증가가 성체 V1 가소성을 막고, 제거 시 니코틴성 ACh 수용체 신호가 강화되어 가소성 재개; "Lynx1 expression thus maintains stability of mature cortical networks in the presence of cholinergic innervation" — [Morishita, Miwa, Heintz, Hensch 2010 Science](https://doi.org/10.1126/science.1195320); perineuronal net을 성체에서 제거하면 임계기 가소성이 재활성화; 구조적 브레이크에 PNN, Nogo 수용체 매개 수초 억제 신호, PirB 포함(검색 요약) — [Perineuronal Nets and the Control of Plasticity](https://pdfs.semanticscholar.org/fa2b/da7ea29b434ec5d3f17d5dbbe72f89f8029b.pdf); 성체 V1에 이식한 PV 뉴런에 PNN이 조기 침착 — [Sci Rep 2018](https://www.nature.com/articles/s41598-018-25735-8); 산화 스트레스와 임계기 이상(조현병 궤적) — [Schizophr Bull 2015 (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4466197/)
- **인간의 위계적 임계기(2025/26 리뷰).** 동물의 1차 피질 임계기가 인간에서는 아동·청소년기에 "unfolding hierarchically from sensorimotor to association regions"일 가능성; 임계기 조절자는 시상피질 입력 발달 → 흥분 활동 → PV 중간뉴런 신호 → 피라미드 연결 → 피질내 수초 → PNN의 순차 발달; 전두엽의 Lynx1 발현 증가가 PNN 밀도 증가와 일치해 "posterior-to-anterior developmental gradient"를 시사(검색 요약) — [Sydnor et al., Investigating hierarchical critical periods in human neurodevelopment, Neuropsychopharmacology](https://www.nature.com/articles/s41386-025-02246-5)

### Inferences
- 설계 원리 15 — **성장-가지치기 스케줄.** 모듈을 과잉 파라미터로 시작해(과잉생성) 사용률·기여도 낮은 유닛/연결을 주기적으로 제거(가지치기)하고, Dohare 2024의 재초기화와 결합하면 "용량 유지 + 가소성 유지"를 동시에 얻는다.
- 설계 원리 16 — **위계적 임계기 커리큘럼.** 감각/전처리 모듈 → 연합/세계모델 → 제어/계획 모듈 순으로 **학습률 창을 열고 닫아라**. 하위 모듈이 안정되기 전에 상위를 훈련하면 상위는 움직이는 표적을 쫓는다. 창을 닫을 때는 (a) 학습률 축소(수초화·PNN의 소프트웨어 대응), (b) 콜린성 브레이크처럼 "놀라움 신호가 임계값을 넘을 때만" 재개방하는 게이트를 둔다.
- 왜 후방→전방인가에 대한 계산적 해석: 전방(전전두) 모듈의 입력은 후방 모듈의 출력이므로, 입력 통계가 안정된 뒤 학습해야 표현이 수렴한다. 또한 가장 늦게 닫히는 전전두가 가장 오래 가소적이라는 점은 **"최상위 제어 모듈은 평생 느리게 계속 학습"**하라는 설계 지침이다.

### Gaps
- 임계기 폐쇄가 "왜 후방부터인가"의 **인과** 기전(유전자 발현 구배 vs 활동 의존)은 아직 연구 중이며 확정 자료를 찾지 못했다.
- Huttenlocher의 원 데이터(영역별 시냅스 밀도 곡선)는 2차 리뷰(PMC3055433)로만 확인했다.

---

## 10. 종합: 소비자 PC/소형 장치용 소프트웨어에 옮길 설계 원칙, 뇌 모사 AI가 이득을 재현하지 못한 지점, Thousand Brains 평가

### Takeaway
전이 가능한 것은 **조직 원리**(선천 스캐폴드/학습 내용 분리, 두 저장소+우선순위 리플레이, 조절 벡터로 게이팅되는 가소성, 전방모델, 직렬 전역 작업공간, 희소·국소·이벤트 구동, 성장-가지치기-임계기 스케줄)이고, 전이에 실패한 것은 **학습 알고리즘 자체의 대체**(PC/FF/Hebbian으로 backprop 대체, 능동추론으로 RL 대체, 스파이킹으로 GPU 대체)이다. Thousand Brains 이론은 "피질 기둥 = 참조 프레임을 가진 독립 학습 모듈 + 투표"라는 유용한 공학 가설이지만, 모든 기둥에 격자세포가 있다는 신경과학적 주장은 논쟁적이고 2025년 기준 Monty 구현의 공개 벤치마크는 제한적이다.

### Cited Findings
- **Thousand Brains Project.** 학습 모듈은 "semi-independent unit that can model entire objects"이고 "represents information through spatially structured reference frames"; 모듈 간 "cortical messaging protocol"로 위계·비위계 표현과 다중감각 통합; 학습은 "similar to Hebbian learning in the brain"; Monty는 "early version"이며 초록에 벤치마크 수치 없음 — [TBP arXiv 2412.18354](https://arxiv.org/abs/2412.18354); 2025년 후속: 빠르고 강건한 학습·추론을 주장하는 Thousand-Brains Systems — [arXiv 2507.04494](https://arxiv.org/pdf/2507.04494), 장거리 연결 확장 TBT 2.0 — [arXiv 2507.05888](https://arxiv.org/pdf/2507.05888); 2025년 1월 Numenta에서 독립 비영리로 분사(검색 요약); 생물학적 제약 AI로서의 평가 — [Hole & Ahmad 2021](https://link.springer.com/article/10.1007/s42452-021-04715-0)
- **TBT 비판.** 모든 피질 기둥이 특징과 위치 둘 다에 접근한다는 것은 "really far-fetched"이며 위치 표상은 내후각피질, 특징-위치 결합은 해마의 역할이라는 반론; "a small patch of entorhinal cortex" 이상을 설명하는지 의문; 피질 "기둥"의 정의 자체가 비일관(종 간 존재·성격이 불일치) — [HTM Forum: Is Thousand Brains Theory wrong?](https://discourse.numenta.org/t/is-thousand-brains-theory-wrong/9231); 미니컬럼 가설에 비춘 모듈 반복의 ML 분석 — [arXiv 2507.12473](https://arxiv.org/pdf/2507.12473). Numenta 측은 전전두·V1·S1에서 격자세포 보고를 근거로 반박(검색 요약).
- 위 1~9번의 모든 "AI 미검증/실패" 항목: PC 확장성 — [PCX](https://arxiv.org/abs/2407.01163); AIF=RL 동형·EFE 이점 부재 — [Millidge 회고](https://www.beren.io/2024-07-27-A-Retrospective-on-Active-Inference/); FF 소규모 — [Hinton 2022](https://arxiv.org/abs/2212.13345); Hebbian ImageNet 27.3% — [SoftHebb](https://arxiv.org/abs/2209.11883); 삼인자 SNN "not tested on large-scale networks" — [arXiv 2504.05341](https://arxiv.org/html/2504.05341v2); 리플레이 비용·가소성 상실 — [van de Ven 2024](https://ar5iv.labs.arxiv.org/html/2403.05175), [Dohare 2024](https://www.nature.com/articles/s41586-024-07711-7); 신경모방 소프트웨어 공백 — [Muir & Sheik 2025](https://www.nature.com/articles/s41467-025-57352-1); PP 증거의 적응 혼입 — [Walsh 2020](https://pmc.ncbi.nlm.nih.gov/articles/PMC7187369/); 뇌 효율의 과장 — [Levy & Calvert 2021](https://www.pnas.org/doi/10.1073/pnas.2008173118)

### Inferences

**(A) 모듈 지도: 뇌 구조 → 기능 → 학습 신호 → 소프트웨어 대응물**

| 뇌 구조 | 기능 | 학습 신호/규칙 | 소프트웨어 대응(소비자 PC) |
|---|---|---|---|
| 뇌간·시상하부 | 항상성·본능·각성 | 거의 학습 없음(호르몬·대사 입력으로 파라미터화) | 내부 상태 변수 + set-point + 반사 정책(코드로 고정) |
| 편도체·내장운동 피질(뇌섬엽·대상) | 가치·정동 = 내수용 예측 | 알로스타시스 예측오차 | 보상 = 예측된 내부 편차 감소; 위험·불확실성 신호 생성 |
| 신경조절핵(VTA/SNc, 기저전뇌, LC, DRN) | 전역 학습 신호 | RPE·기대/예상 못한 불확실성·메타 학습률 | 4채널 "조절 벡터"로 각 모듈 학습률·이득·탐색을 게이팅 |
| 기저핵 | 행동 선택·습관 | 삼인자(DA 게이트) Go/NoGo | 탈억제 WTA 선택기 + Go/NoGo 두 헤드, 맥락 임베딩 입력 |
| 소뇌 | 전방모델·타이밍 | 지도학습(등반섬유 오차) | 효과기별 소형 "명령→결과" 예측기, 예측오차=놀라움 신호 |
| 해마·내후각 | 1회 노출 에피소드·공간/관계 | 희소·pattern separation, 빠른 Hebbian | 벡터 DB/k-NN + 해시 분리 + 우선순위 리플레이 버퍼 |
| 신피질(정준 미세회로 복제) | 느린 통계 학습·예측 | 리플레이 증류, 예측오차, 정밀도 가중 | 동일 템플릿 모듈 다수, 블록-국소 backprop, 하향 예측/상향 오차 채널 |
| 시상·전전두-두정(GNW) | 전역 방송·작업기억 | 점화 임계·경쟁 | 단일 슬롯(≈4 청크) 전역 버퍼, 비선형 경쟁 후 방송, fast-weights 컨텍스트 |
| 수면 | 오프라인 통합·가지치기 | 우선순위 리플레이·예측적 망각 | 유휴 시간 통합 루프: 리플레이→증류→가지치기→재초기화 |

**(B) 구현 순서(권장)** — 1) 내부 상태 변수와 조절 벡터(5번)부터; 2) 두 저장소와 통합 루프(4번); 3) 효과기 전방모델(6번); 4) 전역 작업공간 병목(6번); 5) 희소 조건부 실행과 통신 최소화(8번); 6) 성장-가지치기-임계기 스케줄(9번); 7) 선천 prior를 압축 생성기(g-network형 하이퍼네트워크)로 분리해 "0-step 성능"을 추적(1번).

**(C) 실패 경고(뇌를 베꼈지만 이득이 안 난 곳)** — backprop을 PC/FF/Hebbian으로 바꾸는 것(ImageNet급 실패); RL을 능동추론으로 바꾸는 것(동형, 실무 이득 없음); 스파이킹 하드웨어로 범용 가속(밀집 워크로드 열세, 소프트웨어 공백); 해마 리플레이를 "원시 데이터 재생"으로만 모사(비용·가소성 트레이드오프·용량 포화); 신경조절물질 모사가 대규모 CL 벤치마크에서 SOTA를 넘은 증거 부재; TBT의 "모든 기둥에 위치 표상"을 전제로 한 설계.

**(D) 무엇이 확립이고 무엇이 이론인가(요약)** — 확립: 선천 회로의 존재와 피질 내용의 경험 의존성, 정준 미세회로의 반복, 도파민 위상 반응≈RPE(1차 근사), ACh/NE/5-HT의 불확실성·학습률 역할, 삼인자 가소성과 적격성 흔적, 해마 리플레이와 수면 통합, 소뇌 전방모델, 점화·직렬 병목, 희소 발화와 통신 비용 우위, 후방→전방 발달과 분자 브레이크. 이론/논쟁: 오차 전용 뉴런과 정밀도의 구현(PC), FEP의 형이상학, 도파민의 단일 기능, 기저핵 경로의 정확한 계산 역할, 작업기억의 신경 코드, 모든 피질 기둥의 참조 프레임(TBT), "왜 후방부터 닫히는가".

### Gaps
- Thousand Brains/Monty의 **공개 정량 벤치마크**(객체 인식 정확도, 학습 속도)는 초록 수준에서 확인하지 못했다.
- Hassabis 2017 본문이 제시한 "신경과학이 DQN/EWC에 기여한 구체 경로"는 초록만 확인했다.
- 본 노트는 2026-10-06 기준 WebSearch 예산 소진으로 일부 2025~26년 arXiv 결과(예: 대규모 국소 학습 규칙 비교표, 신경모방 2026 공식 벤치마크)를 추가 탐색하지 못했다.
