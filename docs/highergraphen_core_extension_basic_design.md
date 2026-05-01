# HigherGraphen Core Extension 基本設計

**対象:** `EquivalenceClaim / Derivation / Witness / Scenario / Capability / Policy / Valuation / SchemaMorphism` を中心とする HigherGraphen core 拡張案  
**ステータス:** Draft 0.1  
**作成日:** 2026-05-01  
**目的:** 抽象概念の追加ではなく、AI が検査・比較・更新・拒否・射影できる構造単位として、既存 HigherGraphen core を拡張する。

---

## 0. 要約

本設計は、HigherGraphen に対して次の新しい core extension を追加するための基本設計である。

| 追加対象 | 主な役割 | 解決したい AI 失敗 |
|---|---|---|
| `EquivalenceClaim` | 「同一視してよいか」を、文脈・基準・証拠・損失つきで扱う | 似たものを同じものとして扱う、または同じものを別物として扱う |
| `Derivation` | 前提から結論への導出過程を扱う | 根拠があることと、正当に導出されたことを混同する |
| `Witness` | 主張・導出・同値性・制約違反などを支える観測可能な証人を扱う | 主張の支持構造が曖昧になる |
| `Scenario` | 仮説世界・到達可能世界・反事実的構造を扱う | 未確認・可能・到達可能・現実成立を混同する |
| `Capability` | actor がどの対象にどの操作を行えるかを扱う | AI が実行不可・承認不可の操作を fact として進める |
| `Policy` | 許可・禁止・要求・レビュー条件を構造化する | 権限、レビュー、外部共有、安全条件が暗黙化する |
| `Valuation` | 構造、変換、候補、シナリオへの価値付け・順序付けを扱う | 「正しい」と「望ましい」を混同する |
| `SchemaMorphism` | ontology / schema / interpretation package の変更対応を扱う | model 自体の変化により過去構造の意味が崩れる |

ただし、本設計は `Locality`、`Transformability`、`Invariance`、`Obstruction`、`Interpretation` のような抽象名詞を新しい object type として追加しない。これらは既存の `Context`、`Morphism`、`Invariant`、`Obstruction`、`InterpretationPackage` にすでに近い責務があるため、重複する object type を増やすのではなく、既存概念の演算・検査・証明・診断を強化する。

---

## 1. 背景

HigherGraphen は、通常のグラフを超えて、`Space / Cell / Complex / Context / Morphism / Invariant / Obstruction / CompletionCandidate / Projection / InterpretationPackage` などを AI agent が操作可能な product-level object として扱うことを目指す基盤である。

既存の core concept はすでに以下を含んでいる。

| 既存概念 | 既存の責務 |
|---|---|
| `Space` | 対象世界を収める最上位構造コンテナ |
| `Cell` | entity、relation、observation point、constraint、higher-order relation などを表す typed structural element |
| `Complex` | cells と incidences からなる構造体 |
| `Context` | 意味、妥当性、語彙、規則が成立する局所領域 |
| `Section` | context 上の assignment。局所観測、局所解釈、context-specific value を表す |
| `Morphism` | abstraction、refinement、translation、projection、migration、interpretation などの構造写像 |
| `Invariant` | 特定の scope や transformation の下で保存されるべき性質 |
| `Constraint` | cells、contexts、assignments に対する checkable condition |
| `Obstruction` | 条件、変換、global assembly が安全に成立しない理由 |
| `CompletionCandidate` | 欠けている構造の reviewable candidate |
| `Projection` | audience と purpose に応じた view。情報損失を明示する |
| `InterpretationPackage` | domain meaning を abstract HigherGraphen model に対応させる layer |
| `Provenance` | 構造要素の由来、抽出方法、confidence、review status |

したがって、今回の拡張では「抽象度の高い言葉を増やす」ことではなく、既存 core がまだ明示的に扱いきれていない次の問いを扱う。

```text
同一視してよいか。
根拠から結論へ、どの規則で正当化されたか。
この構造は現実世界ではなく、どの仮説世界で成立するのか。
誰がどの構造変換を実行・承認できるのか。
どの価値順序の下で望ましいと判断されたのか。
ontology / schema / interpretation package 自体が変化したとき、既存構造をどう移行するのか。
```

---

## 2. 設計原則

### 2.1 抽象名詞をそのまま object type にしない

`Locality`、`Composition`、`Invariance`、`Transformability` のような言葉は、設計上の観察軸としては有用だが、そのまま object type にすると責務が曖昧になる。

悪い例:

```text
Locality
Composition
Invariance
Transformability
Interpretation
```

良い例:

```text
ContextOverlap
GluingAttempt
CompositionTrace
InvariantPreservationCheck
MorphismContract
SemanticPreservationCheck
```

本設計では、抽象語そのものではなく、検査・比較・承認・拒否・射影が可能な object を追加する。

### 2.2 既存 core と重複させない

新概念は、既存 core object の代替ではない。

| 追加しない概念 | 理由 | 強化する既存概念 |
|---|---|---|
| `Locality` | `Context` と `Section` が担っている | `ContextOverlap`, `Restriction`, `GluingAttempt` |
| `Transformability` | `Morphism` が担っている | `MorphismContract`, `CompositionTrace`, `RecoverabilityCheck` |
| `Invariance` | `Invariant` が担っている | `InvariantPreservationCheck`, `ViolationWitness` |
| `Obstructionality` | `Obstruction` が担っている | `MinimalUnsatCore`, `ResolutionPath` |
| `Interpretation` | `InterpretationPackage` と `Morphism` が担っている | `SemanticPreservationCheck`, `ConceptAlignment` |

### 2.3 candidate と accepted を分離する

AI が生成した構造は、初期状態では accepted fact ではない。これは `CompletionCandidate` に限らず、今回追加する object すべてに適用する。

```text
EquivalenceClaim は accepted equivalence ではない。
Scenario は現実世界ではない。
Valuation は客観的価値ではない。
PolicyInterpretation は承認済み policy ではない。
Derivation は verifier を通るまで proof ではない。
```

### 2.4 すべての新 object は操作意味論を持つ

新しい概念は、次の少なくとも一部を持つ必要がある。

```text
data model
engine operation
validation check
projection
review workflow
agent-facing operation
failure mode
```

これらを定義できない概念は、core object ではなく annotation、derived report、projection field、または interpretation package 側の domain concept として扱う。

### 2.5 domain product では domain language に戻す

core では高度な構造を扱ってよいが、human-facing projection では domain language に戻す。

悪い projection:

```text
この結果は non-faithful morphism 下の local-to-global gluing obstruction である。
```

良い projection:

```text
Sales 文脈の Customer と Billing 文脈の Customer を同一視すると、請求先責任主体の区別が失われます。
この同一視は、境界を越えた意味損失を伴うため、承認なしに統合できません。
```

---

## 3. アーキテクチャ概要

### 3.1 レイヤ構造

```text
Domain Product Layer
  - Architecture Review
  - CaseGraphen
  - Contract Review
  - Incident Analysis
  - Research Synthesis
          |
          v
Interpretation Layer
  - InterpretationPackage
  - Domain Vocabulary
  - Domain Projection Templates
  - Domain Completion Rules
          |
          v
Core Structural Layer
  - Space / Cell / Complex / Context / Section
  - Morphism / Invariant / Constraint / Obstruction
  - CompletionCandidate / Projection / Provenance
          |
          v
Core Extension Layer
  - EquivalenceClaim
  - Derivation / Witness
  - Scenario
  - Capability / Policy
  - Valuation
  - SchemaMorphism
```

### 3.2 追加 object の責務分布

| Object | 主な接続先 | 主な projection |
|---|---|---|
| `EquivalenceClaim` | `Cell`, `Context`, `Morphism`, `Projection`, `Provenance` | identity risk report, quotient preview |
| `Derivation` | `Claim-like Cell`, `Evidence Cell`, `Witness`, `InferenceRule`, `Provenance` | audit trail, unsupported inference report |
| `Witness` | `Cell`, `Constraint`, `Invariant`, `Obstruction`, `Derivation` | evidence summary, counterexample report |
| `Scenario` | `Space`, `Context`, `Morphism`, `Invariant`, `Obstruction`, `Valuation` | what-if report, reachability report |
| `Capability` | `Actor Cell`, `Operation`, `Policy`, `Context`, `Target` | agent permission view |
| `Policy` | `Capability`, `Projection`, `CompletionCandidate`, `Scenario` | policy compliance report |
| `Valuation` | `Scenario`, `Morphism`, `CompletionCandidate`, `Projection`, `Obstruction` | decision comparison, trade-off report |
| `SchemaMorphism` | `InterpretationPackage`, `Schema`, `Morphism`, `Projection` | migration report, compatibility report |

---

## 4. 新規 core object 設計

## 4.1 `EquivalenceClaim`

### 4.1.1 目的

`EquivalenceClaim` は、二つ以上の構造要素を「同一視してよい」とする主張を、文脈・基準・証人・反証・損失つきで扱う object である。

これは単なる entity resolution ではない。重要なのは、同一視によってどの区別が失われ、どの invariant や context boundary に影響するかを明示することにある。

### 4.1.2 基本フィールド

```yaml
EquivalenceClaim:
  id: equivalence_claim:<id>
  subjects:
    - ref: cell:<id>
    - ref: cell:<id>
  equivalence_kind:
    enum:
      - strict_identity
      - contextual_equivalence
      - observational_equivalence
      - behavioral_equivalence
      - semantic_near_equivalence
      - quotient_equivalence
  scope:
    contexts:
      - context:<id>
    valid_under_morphisms:
      - morphism:<id>
  criterion:
    description: string
    required_invariants:
      - invariant:<id>
    ignored_distinctions:
      - description: string
  witnesses:
    - witness:<id>
  counter_witnesses:
    - witness:<id>
  quotient_effect:
    lost_distinctions:
      - description: string
    merged_cells:
      - cell:<id>
    affected_invariants:
      - invariant:<id>
    affected_projections:
      - projection:<id>
  confidence: number
  status:
    enum:
      - candidate
      - under_review
      - accepted
      - rejected
      - superseded
  provenance: provenance:<id>
  review:
    required: boolean
    reviewer: actor:<id>
    decision_reason: string
```

### 4.1.3 操作

```text
claim_equivalent(subjects, scope, criterion, witnesses)
claim_distinct(subjects, distinguishing_witness)
check_equivalence(claim_id)
preview_quotient(claim_id)
accept_equivalence(claim_id, review_decision)
reject_equivalence(claim_id, reason)
split_equivalence(claim_id, counter_witness)
project_identity_risk(claim_id, audience, purpose)
```

### 4.1.4 validation rules

```text
EQ-001: scope がない equivalence claim は accepted にできない。
EQ-002: criterion がない equivalence claim は accepted にできない。
EQ-003: strict_identity は counter_witness が存在する場合 accepted にできない。
EQ-004: contextual_equivalence は context 外へ自動拡張してはならない。
EQ-005: quotient_effect が未計算の場合、merge operation を実行してはならない。
EQ-006: affected_invariants に unresolved obstruction がある場合、accepted にできない。
EQ-007: AI-generated equivalence は explicit review なしに accepted にできない。
```

### 4.1.5 既存 core との関係

| 既存 object | 関係 |
|---|---|
| `Cell` | subjects として参照される |
| `Context` | equivalence の有効範囲を制限する |
| `Morphism` | 同一視が projection / migration / interpretation で保存されるかを検査する |
| `Invariant` | 同一視後も保存すべき性質を定義する |
| `Obstruction` | 同一視できない理由を表す |
| `Projection` | quotient preview や identity risk report を生成する |
| `Provenance` | 同一視主張の由来を記録する |

---

## 4.2 `Derivation`

### 4.2.1 目的

`Derivation` は、前提から結論への導出過程を構造化する object である。`Evidence` や `Provenance` は「どこから来たか」「何が観測されたか」を扱うが、`Derivation` は「なぜその結論が許されるか」を扱う。

これにより、次を区別できる。

```text
observed
claimed
inferred
derived
verified
accepted
refuted
unsupported
```

### 4.2.2 基本フィールド

```yaml
Derivation:
  id: derivation:<id>
  conclusion: cell:<id>
  premises:
    - cell:<id>
  inference_rule:
    id: inference_rule:<id>
    name: string
    rule_scope:
      contexts:
        - context:<id>
      interpretation_package: interpretation_package:<id>
  warrants:
    - witness:<id>
  excluded_premises:
    - cell:<id>
  counterexamples:
    - witness:<id>
  verifier:
    kind:
      enum:
        - human_review
        - schema_validator
        - proof_checker
        - test_run
        - static_analysis
        - custom_engine
    ref: string
  verification_status:
    enum:
      - unverified
      - machine_checked
      - human_reviewed
      - failed
      - superseded
  failure_mode:
    enum:
      - missing_premise
      - invalid_rule
      - out_of_scope_rule
      - contradicted_by_witness
      - circular_derivation
      - unsupported_jump
      - verifier_unavailable
      - none
  provenance: provenance:<id>
  review_status:
    enum:
      - candidate
      - under_review
      - accepted
      - rejected
```

### 4.2.3 操作

```text
derive(conclusion, premises, inference_rule)
attach_warrant(derivation_id, witness_id)
check_derivation(derivation_id)
find_unsupported_inference(space_id)
find_circular_derivations(space_id)
downgrade_to_candidate(conclusion_id, reason)
project_audit_trail(conclusion_id, audience, purpose)
```

### 4.2.4 validation rules

```text
DER-001: conclusion は premise と同一 cell だけから循環導出してはならない。
DER-002: inference_rule の rule_scope 外で derivation を accepted にしてはならない。
DER-003: contradicted_by_witness が未解消の場合、verification_status を machine_checked / human_reviewed にできない。
DER-004: unsupported_jump が存在する derivation は accepted にできない。
DER-005: AI-generated derivation は verifier または explicit review なしに accepted にできない。
```

---

## 4.3 `Witness`

### 4.3.1 目的

`Witness` は、claim、derivation、equivalence、constraint violation、invariant preservation、obstruction などを支える観測可能・検査可能な支持構造である。

`Witness` は evidence の一種として扱えるが、より広く「この構造判断を支える具体的な証人」という役割を持つ。

### 4.3.2 基本フィールド

```yaml
Witness:
  id: witness:<id>
  witness_type:
    enum:
      - observation
      - log_entry
      - metric_point
      - test_result
      - code_location
      - document_excerpt
      - counterexample
      - human_review
      - machine_check_result
      - external_reference
  supports:
    - ref: cell:<id>
    - ref: derivation:<id>
    - ref: equivalence_claim:<id>
    - ref: invariant:<id>
    - ref: obstruction:<id>
  contradicts:
    - ref: cell:<id>
  payload_ref:
    kind: string
    uri: string
  validity_contexts:
    - context:<id>
  observed_at: datetime
  provenance: provenance:<id>
  confidence: number
  review_status:
    enum:
      - candidate
      - accepted
      - rejected
      - deprecated
```

### 4.3.3 操作

```text
attach_witness(target, witness)
attach_counterexample(target, witness)
validate_witness(witness_id)
find_claims_without_witness(space_id)
find_conflicting_witnesses(space_id)
project_evidence_summary(target, audience, purpose)
```

### 4.3.4 validation rules

```text
WIT-001: witness_type に応じた payload_ref が必要である。
WIT-002: validity_contexts が空の witness は global witness と見なしてはならない。
WIT-003: rejected witness を accepted derivation の唯一の warrant にしてはならない。
WIT-004: counterexample witness が accepted の場合、対象 claim / derivation / equivalence は再検査されなければならない。
```

---

## 4.4 `Scenario`

### 4.4.1 目的

`Scenario` は、現実構造とは別に、仮説世界、変更後世界、到達可能世界、反事実的世界を扱うための object である。

これは `Uncertainty` とは異なる。`Uncertainty` は認識状態の不確かさを扱うが、`Scenario` は「どの条件を変えた世界か」「base structure からどう到達するか」「その世界では何が保存・破壊されるか」を扱う。

### 4.4.2 基本フィールド

```yaml
Scenario:
  id: scenario:<id>
  base_space: space:<id>
  scenario_kind:
    enum:
      - hypothetical
      - reachable
      - blocked
      - counterfactual
      - planned
      - refuted
      - accepted_operational_plan
  assumptions:
    - cell:<id>
  changed_structures:
    added:
      - cell:<id>
    removed:
      - cell:<id>
    modified:
      - morphism:<id>
  reachable_from:
    ref: space:<id>
    via_morphisms:
      - morphism:<id>
  affected_invariants:
    - invariant:<id>
  expected_obstructions:
    - obstruction:<id>
  required_witnesses:
    - witness:<id>
  valuations:
    - valuation:<id>
  status:
    enum:
      - draft
      - candidate
      - under_review
      - reachable
      - blocked
      - refuted
      - accepted
  provenance: provenance:<id>
  review_status:
    enum:
      - candidate
      - under_review
      - accepted
      - rejected
```

### 4.4.3 操作

```text
create_scenario(base_space, assumptions, changes)
check_reachability(scenario_id)
check_invariants_under_scenario(scenario_id)
detect_scenario_obstructions(scenario_id)
compare_scenarios(scenario_ids, valuation_context)
accept_scenario_as_plan(scenario_id, review_decision)
project_what_if_report(scenario_id, audience, purpose)
```

### 4.4.4 validation rules

```text
SCN-001: scenario は base_space を持たなければならない。
SCN-002: scenario_kind=hypothetical のものを現実の accepted fact として扱ってはならない。
SCN-003: reachable とするには reachability check が必要である。
SCN-004: accepted_operational_plan とするには Capability / Policy check が必要である。
SCN-005: affected_invariants が未検査の場合、safe scenario と表示してはならない。
```

---

## 4.5 `Capability`

### 4.5.1 目的

`Capability` は、actor がある context において、どの対象にどの operation を行えるかを表す object である。

これは単なる access control ではない。HigherGraphen では、AI agent が構造を作成・変更・提案・承認・射影するため、操作可能性そのものを構造に接続する必要がある。

### 4.5.2 基本フィールド

```yaml
Capability:
  id: capability:<id>
  actor: cell:<actor_id>
  operation:
    enum:
      - read
      - propose
      - modify
      - accept
      - reject
      - project
      - execute_morphism
      - merge_equivalence
      - create_scenario
      - approve_policy_exception
  target_type: string
  target_refs:
    - ref: string
  contexts:
    - context:<id>
  preconditions:
    - constraint:<id>
  postconditions:
    - constraint:<id>
  forbidden_effects:
    - description: string
  required_review:
    policy: policy:<id>
    reviewer: cell:<actor_id>
  validity_interval:
    starts_at: datetime
    ends_at: datetime
  provenance: provenance:<id>
  status:
    enum:
      - active
      - suspended
      - expired
      - revoked
      - candidate
```

### 4.5.3 操作

```text
can(actor, operation, target, context)
explain_capability_denial(actor, operation, target, context)
require_review(operation, target, policy)
record_approval(actor, operation, target, review_decision)
revoke_capability(capability_id, reason)
detect_policy_violation(action_trace)
project_agent_permission_view(actor, audience, purpose)
```

### 4.5.4 validation rules

```text
CAP-001: expired / revoked capability を使って構造を変更してはならない。
CAP-002: accept operation は propose operation と分離されなければならない。
CAP-003: AI actor による accepted fact 昇格は explicit policy なしに許可してはならない。
CAP-004: external projection は projection policy を満たさなければならない。
CAP-005: forbidden_effects に該当する morphism は実行不可である。
```

---

## 4.6 `Policy`

### 4.6.1 目的

`Policy` は、許可、禁止、要求、レビュー条件、昇格条件、外部共有条件を構造化する object である。

`Capability` が actor-specific な操作可能性を扱うのに対し、`Policy` は system-wide または context-bound な規則を扱う。

### 4.6.2 基本フィールド

```yaml
Policy:
  id: policy:<id>
  policy_kind:
    enum:
      - permission
      - prohibition
      - obligation
      - review_requirement
      - projection_safety
      - candidate_acceptance
      - data_boundary
      - escalation
  applies_to:
    target_types:
      - string
    contexts:
      - context:<id>
    operations:
      - string
  rule:
    description: string
    constraints:
      - constraint:<id>
  required_witnesses:
    - witness:<id>
  required_derivations:
    - derivation:<id>
  escalation_path:
    - cell:<actor_id>
  violation_obstruction_template: obstruction_template:<id>
  status:
    enum:
      - draft
      - active
      - deprecated
      - revoked
  provenance: provenance:<id>
  review_status:
    enum:
      - candidate
      - accepted
      - rejected
```

### 4.6.3 操作

```text
define_policy(policy)
check_policy(operation, actor, target, context)
explain_policy_violation(policy_id, operation_trace)
require_escalation(policy_id, target)
project_policy_compliance_report(space_id, audience, purpose)
```

### 4.6.4 validation rules

```text
POL-001: active policy は provenance と review_status=accepted を持たなければならない。
POL-002: candidate_acceptance policy なしに CompletionCandidate / EquivalenceClaim / Scenario を accepted にしてはならない。
POL-003: projection_safety policy がある場合、Projection は audience / purpose / information_loss を明示しなければならない。
POL-004: policy conflict がある場合、conflict obstruction を生成しなければならない。
```

---

## 4.7 `Valuation`

### 4.7.1 目的

`Valuation` は、構造、変換、候補、scenario、projection に対する価値付け・順序付け・比較不能性を扱う object である。

これは「正しいか」を扱うものではない。「どの評価文脈の下で、どちらが望ましいか」を扱う。

### 4.7.2 基本フィールド

```yaml
Valuation:
  id: valuation:<id>
  target:
    ref: string
  valuation_context: context:<id>
  criteria:
    - criterion_id: string
      name: string
      direction:
        enum:
          - maximize
          - minimize
          - preserve
          - avoid
      weight: number | null
      required: boolean
  order_type:
    enum:
      - scalar_score
      - lexicographic_order
      - partial_order
      - pareto_frontier
      - threshold_acceptance
      - qualitative_ranking
  values:
    - criterion_id: string
      value: string | number | boolean
      evidence: witness:<id>
  tradeoffs:
    - gains: string
      losses: string
      affected_invariants:
        - invariant:<id>
  incomparable_with:
    - valuation:<id>
  confidence: number
  provenance: provenance:<id>
  review_status:
    enum:
      - candidate
      - under_review
      - accepted
      - rejected
```

### 4.7.3 操作

```text
value(target, context, criteria)
compare(target_a, target_b, valuation_context)
find_pareto_frontier(targets, valuation_context)
explain_tradeoff(valuation_id)
detect_incomparable_values(valuations)
project_decision_matrix(targets, audience, purpose)
```

### 4.7.4 validation rules

```text
VAL-001: valuation_context がない valuation を global value として扱ってはならない。
VAL-002: scalar_score のみによる自動意思決定は policy で許可される場合に限る。
VAL-003: required criterion を満たさない target を best と表示してはならない。
VAL-004: incomparable_with が存在する場合、単一ランキングとして projection してはならない。
VAL-005: valuation は invariant preservation check の代替ではない。
```

---

## 4.8 `SchemaMorphism`

### 4.8.1 目的

`SchemaMorphism` は、schema、ontology、interpretation package、domain vocabulary の変更を扱う morphism の特殊化である。

HigherGraphen では対象構造だけでなく、対象構造を解釈する model 自体が進化する。`SchemaMorphism` は、model 自体の変化によって既存構造がどう再解釈・移行・失効するかを明示する。

### 4.8.2 基本フィールド

```yaml
SchemaMorphism:
  id: schema_morphism:<id>
  source_schema: schema:<id>
  target_schema: schema:<id>
  source_interpretation_package: interpretation_package:<id>
  target_interpretation_package: interpretation_package:<id>
  mapping_kind:
    enum:
      - rename
      - split
      - merge
      - refinement
      - abstraction
      - deprecation
      - semantic_redefinition
      - custom
  mappings:
    - source_ref: string
      target_ref: string
      mapping_rule: string
      preservation_claims:
        - invariant:<id>
      loss_claims:
        - description: string
      required_reviews:
        - policy:<id>
  affected_objects:
    - ref: string
  compatibility:
    enum:
      - backward_compatible
      - forward_compatible
      - lossy
      - incompatible
      - unknown
  verification:
    checks:
      - derivation:<id>
      - witness:<id>
  provenance: provenance:<id>
  review_status:
    enum:
      - candidate
      - under_review
      - accepted
      - rejected
```

### 4.8.3 操作

```text
define_schema_morphism(source_schema, target_schema, mappings)
check_schema_compatibility(schema_morphism_id)
preview_schema_migration(schema_morphism_id, space_id)
find_invalidated_derivations(schema_morphism_id)
find_semantic_drift(schema_morphism_id)
apply_schema_morphism(schema_morphism_id, review_decision)
project_migration_report(schema_morphism_id, audience, purpose)
```

### 4.8.4 validation rules

```text
SCH-001: semantic_redefinition は explicit review なしに accepted にできない。
SCH-002: split / merge mapping は affected_objects と loss_claims を持たなければならない。
SCH-003: lossy schema morphism は projection に loss を明示しなければならない。
SCH-004: incompatible schema morphism を migration として実行してはならない。
SCH-005: accepted derivation の inference_rule が schema morphism により invalidated された場合、derivation を再検査しなければならない。
```

---

## 5. 既存 core の強化方針

## 5.1 `Context` 強化

追加する補助 object / operation:

```text
ContextOverlap
ContextBoundary
Restriction
Extension
GluingAttempt
GluingObstruction
```

目的:

```text
局所的には妥当だが、大域的に統合できない構造を検査する。
context-specific term を無根拠に global meaning へ潰さない。
```

## 5.2 `Morphism` 強化

追加する補助 object / operation:

```text
MorphismContract
CompositionTrace
PreservationCheck
RecoverabilityCheck
DistortionMetric
```

目的:

```text
変換が何を保存し、何を失い、何を歪め、何を復元可能に残すかを検査する。
```

## 5.3 `Invariant` 強化

追加する補助 object / operation:

```text
InvariantScope
InvariantStrength
InvariantPreservationCheck
InvariantViolationWitness
```

目的:

```text
invariant がどこで、どの強さで、どの変換に対して保存されるべきかを明示する。
```

## 5.4 `Obstruction` 強化

追加する補助 object / operation:

```text
ObstructionClass
ObstructionDependency
MinimalUnsatCore
ResolutionCandidate
CounterexampleProjection
```

目的:

```text
なぜ成立しないのかを、最小原因、依存関係、反例、解消候補として扱う。
```

## 5.5 `Projection` 強化

追加する補助 object / operation:

```text
AudienceSafety
LossSemantics
RedactionPolicy
ProjectionContract
```

目的:

```text
projection が誰向けで、何を失い、何を隠し、何を出してはいけないかを検査する。
```

## 5.6 `InterpretationPackage` 強化

追加する補助 object / operation:

```text
SemanticPreservationCheck
ConceptAlignment
SemanticDriftDetection
VocabularyConflict
SchemaMorphism
```

目的:

```text
domain meaning と structural core の対応を検査可能にする。
```

---

## 6. Agent-facing operation contract

今回の拡張により、agent-facing operation は以下を追加する。

| Operation | Purpose |
|---|---|
| `claim_equivalent` | 二つ以上の構造要素の同一視候補を作成する |
| `check_equivalence` | equivalence claim の妥当性、scope、counter witness、quotient effect を検査する |
| `preview_quotient` | 同一視した場合に失われる区別と影響を表示する |
| `derive` | 前提・規則・warrant から導出候補を作る |
| `check_derivation` | 導出規則、scope、反例、循環、unsupported jump を検査する |
| `attach_witness` | claim / derivation / obstruction などに witness を紐づける |
| `create_scenario` | base structure から仮説世界を作る |
| `check_reachability` | scenario が base structure から到達可能か検査する |
| `can` | actor が operation を target に対して実行可能か確認する |
| `check_policy` | operation trace が policy に違反しないか確認する |
| `value` | target に valuation を付与する |
| `compare` | 複数 target を valuation context の下で比較する |
| `define_schema_morphism` | schema / interpretation package の変化を構造化する |
| `preview_schema_migration` | schema morphism 適用時の影響を事前表示する |

すべての create / modify / accept 系 operation は、`provenance` と `review_status` を受け取る必要がある。

---

## 7. 状態遷移

### 7.1 共通 lifecycle

```text
candidate
  -> under_review
    -> accepted
    -> rejected
  -> superseded
```

### 7.2 accepted に必要な条件

| Object | accepted 条件 |
|---|---|
| `EquivalenceClaim` | scope、criterion、witness、quotient effect、review decision がある |
| `Derivation` | inference rule が scope 内で有効、unsupported jump がない、verifier または review がある |
| `Witness` | payload が検査可能、validity context が明示されている |
| `Scenario` | scenario_kind に応じた reachability / policy / invariant check が完了している |
| `Capability` | policy と review により有効化されている |
| `Policy` | provenance と review により承認済みである |
| `Valuation` | valuation context、criteria、evidence がある。比較不能性がある場合は明示されている |
| `SchemaMorphism` | compatibility check、loss claim、affected object、review がある |

---

## 8. Projection 設計

### 8.1 Human-facing projection

human-facing projection では、理論語を前面に出さない。

例:

```text
Sales Customer と Billing Customer は同じ名前ですが、請求責任主体と購買行動主体という異なる意味を持っています。
この2つを統合すると、Billing 側の責任主体の区別が失われます。
したがって、この同一視は候補として保持され、承認なしに採用できません。
```

### 8.2 Agent-facing projection

agent-facing projection では、operation に必要な構造情報を明示する。

```yaml
identity_risk_projection:
  equivalence_claim: equivalence_claim:sales_customer_billing_customer
  status: candidate
  missing:
    - quotient_effect.review
    - billing_context.witness
  blockers:
    - obstruction:semantic_loss_across_billing_boundary
  safe_operations:
    - attach_witness
    - reject_equivalence
    - request_review
  unsafe_operations:
    - merge_equivalence
    - promote_to_accepted_fact
```

### 8.3 Audit projection

監査向け projection では、次を必須にする。

```text
source structures
provenance
review status
derivations
witnesses
policy checks
projection loss
accepted / candidate / rejected の区別
```

---

## 9. Failure modes

| Failure mode | 防止・検出する object |
|---|---|
| 文脈固有の語を global meaning に潰す | `EquivalenceClaim`, `Context`, `Policy` |
| AI inference を accepted fact として扱う | `Derivation`, `Witness`, `Policy`, `Provenance` |
| 根拠と証明を混同する | `Derivation`, `Witness` |
| 仮説世界を現実世界として扱う | `Scenario` |
| 許可されていない操作を実行する | `Capability`, `Policy` |
| 正しさと望ましさを混同する | `Valuation` |
| schema 変更により過去の判断が壊れる | `SchemaMorphism` |
| projection loss を隠す | `Projection`, `Policy`, `Valuation` |
| scalar score により比較不能な価値を潰す | `Valuation` |

---

## 10. 実装順序

### Phase 0: Schema-only prototype

目的:

```text
新 object の JSON schema を定義し、既存 report に追加できるか検証する。
```

対象:

```text
EquivalenceClaim
Witness
Derivation
```

理由:

```text
AI の同一性誤認、根拠と導出の混同は、早期に価値が出る。
既存 Provenance / CompletionCandidate / Obstruction と接続しやすい。
```

### Phase 1: Identity and Justification engine

追加:

```text
check_equivalence
preview_quotient
check_derivation
find_unsupported_inference
find_conflicting_witnesses
```

成果物:

```text
identity risk report
audit trail projection
unsupported inference report
```

### Phase 2: Scenario and Policy engine

追加:

```text
create_scenario
check_reachability
check_policy
can
record_approval
```

成果物:

```text
what-if report
agent permission view
policy compliance report
```

### Phase 3: Valuation engine

追加:

```text
value
compare
find_pareto_frontier
explain_tradeoff
detect_incomparable_values
```

成果物:

```text
decision matrix
trade-off report
scenario comparison projection
```

### Phase 4: SchemaMorphism engine

追加:

```text
define_schema_morphism
check_schema_compatibility
preview_schema_migration
find_invalidated_derivations
find_semantic_drift
```

成果物:

```text
schema migration report
semantic drift report
compatibility report
```

---

## 11. 非目標

本設計では、以下を非目標とする。

```text
すべての数学的概念を object type 化すること。
単一の universal ontology を作ること。
Valuation を単一スコアに還元すること。
AI agent に accepted fact 昇格権限を無条件に与えること。
Policy を単なる access control として扱うこと。
Scenario を uncertainty の別名として扱うこと。
SchemaMorphism を単なる migration script として扱うこと。
```

---

## 12. Open questions

1. `Derivation` と domain-specific reasoning engine の境界をどこに置くか。
2. `Witness` を `Cell` の subtype とするか、独立 object とするか。
3. `EquivalenceClaim` accepted 後に、実際に cell merge を行うか、quotient view として保持するか。
4. `Capability` / `Policy` を core に入れる範囲をどこまで制限するか。
5. `Valuation` の partial order / Pareto frontier をどの程度 schema として固定するか。
6. `SchemaMorphism` を通常の `Morphism` の subtype として扱うか、別 object として扱うか。
7. 既存 CLI / MCP operation にどの順で追加するか。
8. AI-generated `Derivation` の verifier をどの粒度で扱うか。
9. human review の結果を `Witness` として扱うか、`ReviewDecision` として独立させるか。
10. projection loss と valuation loss をどのように分けるか。

---

## 13. まとめ

本設計の中心は、HigherGraphen に「抽象概念」を追加することではない。

中心は、既存 core と衝突しない形で、AI が誤りやすい次の構造判断を一級化することにある。

```text
同一視してよいか。
根拠から結論へ正当に導出されたか。
どの witness が主張を支えているか。
どの仮説世界で成立するか。
誰がどの操作を実行・承認できるか。
どの価値順序の下で望ましいか。
schema / interpretation package の変更により何が失われるか。
```

したがって、追加する core object は以下に絞る。

```text
EquivalenceClaim
Derivation
Witness
Scenario
Capability
Policy
Valuation
SchemaMorphism
```

そして、以下は新 object としては追加せず、既存 core の演算・検査・診断として強化する。

```text
Context
Morphism
Invariant
Obstruction
Projection
InterpretationPackage
Provenance
```

この方針により、HigherGraphen は ontology bloat を避けながら、AI が構造を安全に操作するための基盤性を高められる。

---

## 参考資料

- CAPHTECH/higher-graphen README: https://github.com/CAPHTECH/higher-graphen
- Core Concepts: https://github.com/CAPHTECH/higher-graphen/blob/main/docs/concepts/core-concepts.md
- Theoretical Foundations: https://github.com/CAPHTECH/higher-graphen/blob/main/docs/concepts/theoretical-foundations.md
- AI Agent Integration: https://github.com/CAPHTECH/higher-graphen/blob/main/docs/specs/ai-agent-integration.md
