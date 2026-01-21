# UI Framework SWOT / 前端框架 SWOT

## 1) Context / 背景
中文：本專案需要一個可本機運行的 UI 儀表板，配合 `merrow_ui` API 服務使用，首版以簡單、可維護、低成本為優先。  
English: The project needs a local UI dashboard working with the `merrow_ui` API. V1 prioritizes simplicity, maintainability, and low cost.

## 2) Options / 方案
- A) Svelte SPA + Rust UI API (axum)
- B) React SPA + Rust UI API (axum)
- C) Vue SPA + Rust UI API (axum)

## 3) SWOT

### A) Svelte
Strengths / 優勢
- 中文：語法精簡、學習成本低、編譯後體積小，適合本機儀表板。  
- English: Compact syntax, low learning curve, small bundles; good for local dashboards.

Weaknesses / 劣勢
- 中文：生態系相對小，第三方元件選擇較少。  
- English: Smaller ecosystem; fewer mature third-party components.

Opportunities / 機會
- 中文：可用更少樣板快速迭代 UI；適合 POC 到 MVP 的節奏。  
- English: Rapid iteration with less boilerplate; fits POC → MVP cadence.

Threats / 威脅
- 中文：團隊若熟悉 React/Vue 需再適應；大型 UI 可能需要更嚴謹的架構規範。  
- English: Team may need ramp-up if more React/Vue oriented; large UIs need stricter discipline.

### B) React
Strengths / 優勢
- 中文：生態系最大、資源多、工具成熟。  
- English: Largest ecosystem, abundant resources, mature tooling.

Weaknesses / 劣勢
- 中文：樣板與狀態管理選型較多，容易增加複雜度。  
- English: More boilerplate and state-management choices can add complexity.

Opportunities / 機會
- 中文：長期擴張與招聘較容易，資源相對充足。  
- English: Easier long-term scaling and hiring due to popularity.

Threats / 威脅
- 中文：小型專案易被過度工程；依賴生態變動風險較高。  
- English: Small projects can be over-engineered; ecosystem churn risk.

### C) Vue
Strengths / 優勢
- 中文：模板語法直覺，學習門檻低，中型專案維護性佳。  
- English: Intuitive template syntax, lower learning curve, good for mid-size apps.

Weaknesses / 劣勢
- 中文：生態系規模與第三方資源介於 React 與 Svelte 之間。  
- English: Ecosystem size and third-party options sit between React and Svelte.

Opportunities / 機會
- 中文：若團隊已有 Vue 經驗，可降低上手成本。  
- English: If the team knows Vue, onboarding is fast.

Threats / 威脅
- 中文：與 React 相比，部分企業級元件或案例較少。  
- English: Fewer enterprise-grade components or references than React.

## 4) Decision / 決策
中文：V1 選擇 A) Svelte SPA，理由是可快速交付、低維護成本、與現有 `merrow_ui` API 整合簡單。  
English: V1 uses A) Svelte SPA for faster delivery, lower maintenance cost, and simple integration with `merrow_ui`.

## 5) Follow-ups / 後續
- 中文：如需多人協作或複雜 UI，評估狀態管理規範與元件庫補強。  
- English: If collaboration or UI complexity grows, standardize state management and add component libraries.
