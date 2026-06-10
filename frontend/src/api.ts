import type {
  DrillingParams,
  DrillingResponse,
  ImportResponse,
  JobRecord,
  JobSummary,
  PartType,
  Problem,
  Solution,
} from './types'

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let msg = `Erreur ${res.status}`
    try {
      const body = await res.json()
      if (body?.error) msg = body.error
    } catch {
      /* ignore */
    }
    throw new Error(msg)
  }
  return res.json() as Promise<T>
}

export async function optimize(problem: Problem): Promise<Solution> {
  const res = await fetch('/api/optimize', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(problem),
  })
  return jsonOrThrow<Solution>(res)
}

export async function importFile(file: File, unit: string): Promise<ImportResponse> {
  const form = new FormData()
  form.append('file', file)
  form.append('unit', unit)
  const res = await fetch('/api/import', { method: 'POST', body: form })
  return jsonOrThrow<ImportResponse>(res)
}

function triggerDownload(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}

export async function downloadTemplate(): Promise<void> {
  const res = await fetch('/api/template.csv')
  if (!res.ok) throw new Error('telechargement du modele impossible')
  triggerDownload(await res.blob(), 'modele_liste_coupe.csv')
}

export async function exportCsv(
  jobNumber: string,
  parts: PartType[],
  solution: Solution,
  unit: string,
): Promise<void> {
  const res = await fetch('/api/export.csv', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ job_number: jobNumber, parts, solution, unit }),
  })
  if (!res.ok) throw new Error('export impossible')
  const name = jobNumber ? `plan_de_coupe_${jobNumber}.csv` : 'plan_de_coupe.csv'
  triggerDownload(await res.blob(), name)
}

export async function computeDrilling(
  parts: PartType[],
  params: DrillingParams,
): Promise<DrillingResponse> {
  const res = await fetch('/api/drilling', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ parts, ...params }),
  })
  return jsonOrThrow<DrillingResponse>(res)
}

export async function exportDrillingCsv(
  parts: PartType[],
  params: DrillingParams,
  unit: string,
): Promise<void> {
  const res = await fetch('/api/drilling.csv', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ parts, ...params, unit }),
  })
  if (!res.ok) throw new Error('export perçage impossible')
  triggerDownload(await res.blob(), 'plan_de_percage.csv')
}

export interface SaveJobPayload {
  job_number: string
  settings: unknown
  stocks: unknown
  parts: unknown
  result: unknown
}

export async function saveJob(payload: SaveJobPayload): Promise<{ id: string }> {
  const res = await fetch('/api/jobs', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  return jsonOrThrow<{ id: string }>(res)
}

export async function listJobs(): Promise<JobSummary[]> {
  const res = await fetch('/api/jobs')
  return jsonOrThrow<JobSummary[]>(res)
}

export async function getJob(id: string): Promise<JobRecord> {
  const res = await fetch(`/api/jobs/${id}`)
  return jsonOrThrow<JobRecord>(res)
}

export async function deleteJob(id: string): Promise<void> {
  const res = await fetch(`/api/jobs/${id}`, { method: 'DELETE' })
  if (!res.ok && res.status !== 204) throw new Error('suppression impossible')
}
