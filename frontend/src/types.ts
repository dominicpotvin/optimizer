// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
export interface StockType {
  id: string
  label: string
  length_um: number
  available: number | null
}

export interface PartType {
  id: string
  label: string
  length_um: number
  qty: number
  model?: string | null
}

export interface Settings {
  kerf_um: number
  reusable_threshold_um: number
  time_limit_ms?: number | null
}

export interface Problem {
  stocks: StockType[]
  parts: PartType[]
  settings: Settings
}

export interface PlacedCut {
  part_id: string
  label: string
  length_um: number
}

export interface BarPlan {
  stock_id: string
  stock_label: string
  stock_length_um: number
  cuts: PlacedCut[]
  used_um: number
  kerf_total_um: number
  offcut_um: number
  reusable: boolean
}

export interface StockUsage {
  stock_id: string
  label: string
  length_um: number
  count: number
}

export interface Summary {
  total_bars: number
  bars_by_stock: StockUsage[]
  total_stock_um: number
  total_parts_um: number
  total_kerf_um: number
  total_offcut_um: number
  reusable_offcut_um: number
  real_waste_um: number
  utilization_pct: number
  reusable_count: number
}

export interface Solution {
  bars: BarPlan[]
  summary: Summary
  complete: boolean
  unplaced: PlacedCut[]
}

export interface JobSummary {
  id: string
  job_number: string
  created_at: string
  status: string
  total_bars: number | null
}

export interface JobRecord {
  id: string
  job_number: string
  created_at: string
  status: string
  settings: Settings
  stocks: StockType[]
  parts: PartType[]
  result: Solution | null
}

export interface ImportItem {
  part_id: string
  model: string | null
  job_id: string | null
  length_um: number
  qty: number
}

export interface ImportResponse {
  count: number
  job_numbers: string[]
  items: ImportItem[]
}

export interface DrillingParams {
  pas_um: number
  marge_min_um: number
  marge_max_um: number
}

export interface Hole {
  index: number
  from_a_um: number
  from_b_um: number
  is_center: boolean
}

export interface DrillResult {
  ok: boolean
  message: string | null
  n: number
  marge_um: number
  pas_um: number
  parity: string
  holes: Hole[]
}

export interface PartDrilling {
  part_id: string
  label: string
  model: string | null
  length_um: number
  qty: number
  result: DrillResult
}

export interface DrillingResponse {
  results: PartDrilling[]
}
