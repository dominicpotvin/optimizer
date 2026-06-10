// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
import { useEffect, useRef, useState } from 'react'
import LengthInput from './LengthInput'
import ResultView from './ResultView'
import DrillingView from './DrillingView'
import History from './History'
import Icon from './Icon'
import SectionHelp from './SectionHelp'
import * as api from './api'
import type { JobSummary, PartDrilling, PartType, Problem, Solution, StockType } from './types'
import {
  fineUnitFor,
  formatLength,
  lengthUnitFor,
  toUm,
  type UnitSystem,
} from './units'

interface StockRow {
  key: string
  label: string
  length_um: number
  available: number | null
}
interface PartRow {
  key: string
  part_id: string
  model: string
  length_um: number
  qty: number
}

let _seq = 0
const uid = () => `r${++_seq}`

function defaultStocks(): StockRow[] {
  return [{ key: uid(), label: '16 pi', length_um: toUm(16, 'pi'), available: null }]
}
function exampleParts(): PartRow[] {
  return [
    { key: uid(), part_id: 'P1', model: 'Profil-U-100', length_um: toUm(6, 'pi'), qty: 2 },
    { key: uid(), part_id: 'P2', model: 'Profil-U-100', length_um: toUm(5, 'pi'), qty: 6 },
    { key: uid(), part_id: 'P3', model: 'Profil-U-100', length_um: toUm(4, 'pi'), qty: 1 },
    { key: uid(), part_id: 'P4', model: 'Profil-U-100', length_um: toUm(3, 'pi'), qty: 1 },
  ]
}

type Msg = { kind: 'err' | 'ok' | 'warn'; text: string } | null

export default function App() {
  const [system, setSystem] = useState<UnitSystem>('imperial')
  const [jobNumber, setJobNumber] = useState('')
  const [stocks, setStocks] = useState<StockRow[]>(defaultStocks)
  const [parts, setParts] = useState<PartRow[]>(exampleParts)
  const [kerfUm, setKerfUm] = useState(toUm(0.125, 'po'))
  const [thresholdUm, setThresholdUm] = useState(toUm(1, 'pi'))
  // Perçage (défauts du cahier des charges : entraxe 112 mm, marges 40–112 mm)
  const [pasUm, setPasUm] = useState(toUm(112, 'mm'))
  const [margeMinUm, setMargeMinUm] = useState(toUm(40, 'mm'))
  const [margeMaxUm, setMargeMaxUm] = useState(toUm(112, 'mm'))
  const [drillResults, setDrillResults] = useState<PartDrilling[] | null>(null)
  const [solution, setSolution] = useState<Solution | null>(null)
  const [msg, setMsg] = useState<Msg>(null)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [jobs, setJobs] = useState<JobSummary[]>([])
  const [dbAvailable, setDbAvailable] = useState(true)
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    try {
      const saved = localStorage.getItem('theme')
      if (saved === 'light' || saved === 'dark') return saved
    } catch {
      /* ignore */
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  })
  const fileRef = useRef<HTMLInputElement>(null)

  const lenUnit = lengthUnitFor(system)
  const fineUnit = fineUnitFor(system)

  useEffect(() => {
    refreshJobs()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark')
    try {
      localStorage.setItem('theme', theme)
    } catch {
      /* ignore */
    }
  }, [theme])

  function buildProblem(): { problem: Problem; apiParts: PartType[]; apiStocks: StockType[] } | null {
    const apiStocks: StockType[] = stocks
      .filter((s) => s.length_um > 0)
      .map((s, i) => ({
        id: s.label.trim() || `stock${i + 1}`,
        label: s.label.trim() || formatLength(s.length_um, system),
        length_um: s.length_um,
        available: s.available,
      }))
    const apiParts: PartType[] = parts
      .filter((p) => p.length_um > 0 && p.qty > 0)
      .map((p, i) => ({
        id: p.part_id.trim() || `P${i + 1}`,
        label: p.part_id.trim() || formatLength(p.length_um, system),
        length_um: p.length_um,
        qty: Math.max(1, Math.round(p.qty)),
        model: p.model.trim() || null,
      }))

    if (apiStocks.length === 0) {
      setMsg({ kind: 'err', text: 'Ajoutez au moins une longueur de stock.' })
      return null
    }
    if (apiParts.length === 0) {
      setMsg({ kind: 'err', text: 'Ajoutez au moins une piece a couper.' })
      return null
    }
    const problem: Problem = {
      stocks: apiStocks,
      parts: apiParts,
      settings: {
        kerf_um: kerfUm,
        reusable_threshold_um: thresholdUm,
        time_limit_ms: 800,
      },
    }
    return { problem, apiParts, apiStocks }
  }

  function drillParams() {
    return { pas_um: pasUm, marge_min_um: margeMinUm, marge_max_um: margeMaxUm }
  }

  async function runOptimize() {
    const built = buildProblem()
    if (!built) return
    setLoading(true)
    setMsg(null)
    try {
      const [sol, drill] = await Promise.all([
        api.optimize(built.problem),
        api.computeDrilling(built.apiParts, drillParams()),
      ])
      setSolution(sol)
      setDrillResults(drill.results)
      if (!sol.complete) {
        setMsg({ kind: 'warn', text: `${sol.unplaced.length} piece(s) non placee(s).` })
      }
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message ?? 'Erreur de calcul' })
    } finally {
      setLoading(false)
    }
  }

  async function doExportDrilling() {
    const built = buildProblem()
    if (!built) return
    try {
      await api.exportDrillingCsv(built.apiParts, drillParams(), lenUnit)
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message })
    }
  }

  async function doImport(file: File) {
    setMsg(null)
    try {
      const res = await api.importFile(file, lenUnit)
      setParts(
        res.items.map((it) => ({
          key: uid(),
          part_id: it.part_id,
          model: it.model ?? '',
          length_um: it.length_um,
          qty: it.qty,
        })),
      )
      if (!jobNumber && res.job_numbers.length === 1) setJobNumber(res.job_numbers[0])
      setMsg({ kind: 'ok', text: `${res.count} ligne(s) importee(s).` })
      setSolution(null)
      setDrillResults(null)
    } catch (e: any) {
      setMsg({ kind: 'err', text: `Import : ${e.message}` })
    }
  }

  async function doExport() {
    const built = buildProblem()
    if (!built || !solution) return
    try {
      await api.exportCsv(jobNumber, built.apiParts, solution, lenUnit)
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message })
    }
  }

  async function doSave() {
    const built = buildProblem()
    if (!built) return
    setSaving(true)
    setMsg(null)
    try {
      await api.saveJob({
        job_number: jobNumber,
        settings: { ...built.problem.settings, drilling: drillParams() },
        stocks: built.apiStocks,
        parts: built.apiParts,
        result: solution,
      })
      setMsg({ kind: 'ok', text: 'Job enregistre.' })
      refreshJobs()
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message })
    } finally {
      setSaving(false)
    }
  }

  async function refreshJobs() {
    try {
      const list = await api.listJobs()
      setJobs(list)
      setDbAvailable(true)
    } catch {
      setDbAvailable(false)
    }
  }

  async function loadJob(id: string) {
    try {
      const rec = await api.getJob(id)
      setJobNumber(rec.job_number)
      setStocks(rec.stocks.map((s) => ({ key: uid(), label: s.label, length_um: s.length_um, available: s.available })))
      setParts(
        rec.parts.map((p) => ({
          key: uid(),
          part_id: p.id,
          model: p.model ?? '',
          length_um: p.length_um,
          qty: p.qty,
        })),
      )
      setKerfUm(rec.settings.kerf_um)
      setThresholdUm(rec.settings.reusable_threshold_um)
      const dr = (rec.settings as any).drilling as
        | { pas_um: number; marge_min_um: number; marge_max_um: number }
        | undefined
      if (dr) {
        setPasUm(dr.pas_um)
        setMargeMinUm(dr.marge_min_um)
        setMargeMaxUm(dr.marge_max_um)
      }
      setSolution(rec.result)
      // Recalcule le perçage pour l'affichage.
      api
        .computeDrilling(rec.parts, dr ?? drillParams())
        .then((d) => setDrillResults(d.results))
        .catch(() => setDrillResults(null))
      setMsg({ kind: 'ok', text: `Job « ${rec.job_number} » charge.` })
      window.scrollTo({ top: 0, behavior: 'smooth' })
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message })
    }
  }

  async function removeJob(id: string) {
    try {
      await api.deleteJob(id)
      refreshJobs()
    } catch (e: any) {
      setMsg({ kind: 'err', text: e.message })
    }
  }

  // --- editeurs de lignes ---
  const updStock = (key: string, patch: Partial<StockRow>) =>
    setStocks((rows) => rows.map((r) => (r.key === key ? { ...r, ...patch } : r)))
  const updPart = (key: string, patch: Partial<PartRow>) =>
    setParts((rows) => rows.map((r) => (r.key === key ? { ...r, ...patch } : r)))

  return (
    <div className="wrap">
      <header className="app">
        <div className="brand">
          <img className="brand-logo" src="/LogoGestEase256.ico" alt="GestEase" />
          <div>
            <h1>Optimiseur de coupe</h1>
            <p className="sub">Plans de coupe d'extrusions — minimisation des pertes · GestEase</p>
          </div>
        </div>
        <div className="header-right">
          <div className="seg" role="group" aria-label="systeme d'unites">
            <button className={system === 'imperial' ? 'active' : ''} onClick={() => setSystem('imperial')}>
              Imperial (pi/po)
            </button>
            <button className={system === 'metric' ? 'active' : ''} onClick={() => setSystem('metric')}>
              Metrique (mm)
            </button>
          </div>
          <button
            className="theme-toggle"
            onClick={() => setTheme((t) => (t === 'dark' ? 'light' : 'dark'))}
            title={theme === 'dark' ? 'Passer en mode clair' : 'Passer en mode sombre'}
            aria-label={theme === 'dark' ? 'Passer en mode clair' : 'Passer en mode sombre'}
          >
            <Icon name={theme === 'dark' ? 'light_mode' : 'dark_mode'} />
          </button>
        </div>
      </header>

      {msg && <div className={`banner ${msg.kind}`}>{msg.text}</div>}

      {/* Donnees de base */}
      <div className="card">
        <h2>Donnees de base <SectionHelp topic="base" /></h2>
        <div className="row">
          <div style={{ flex: '1 1 280px' }}>
            <label htmlFor="job">Numero de job (texte ou code-barres)</label>
            <input
              id="job"
              autoFocus
              value={jobNumber}
              placeholder="Scannez ou saisissez le no de job…"
              onChange={(e) => setJobNumber(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') (e.target as HTMLInputElement).blur()
              }}
            />
            <div className="barcode-hint">Compatible lecteur code-barres (USB/clavier) : scannez, le champ se remplit.</div>
          </div>
          <div style={{ flex: '0 1 160px' }}>
            <label>Trait de scie (kerf)</label>
            <LengthInput um={kerfUm} unit={fineUnit} onChange={setKerfUm} />
          </div>
          <div style={{ flex: '0 1 180px' }}>
            <label>Seuil chute reutilisable</label>
            <LengthInput um={thresholdUm} unit={lenUnit} onChange={setThresholdUm} />
          </div>
        </div>
      </div>

      {/* Inventaire de stock */}
      <div className="card">
        <h2>Longueurs de stock disponibles <SectionHelp topic="stock" /></h2>
        <p className="hint">Laissez « dispo » vide pour une quantite illimitee.</p>
        <table>
          <thead>
            <tr>
              <th>Etiquette</th>
              <th>Longueur</th>
              <th>Dispo</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {stocks.map((s) => (
              <tr key={s.key}>
                <td>
                  <input value={s.label} onChange={(e) => updStock(s.key, { label: e.target.value })} />
                </td>
                <td>
                  <LengthInput um={s.length_um} unit={lenUnit} onChange={(v) => updStock(s.key, { length_um: v })} />
                </td>
                <td className="num">
                  <input
                    type="number"
                    min={0}
                    value={s.available ?? ''}
                    placeholder="∞"
                    onChange={(e) =>
                      updStock(s.key, { available: e.target.value === '' ? null : Math.max(0, parseInt(e.target.value) || 0) })
                    }
                  />
                </td>
                <td className="act">
                  <button className="ghost" onClick={() => setStocks((r) => r.filter((x) => x.key !== s.key))} aria-label="Retirer">
                    <Icon name="close" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="toolbar" style={{ marginTop: 10 }}>
          <button
            className="secondary"
            onClick={() =>
              setStocks((r) => [...r, { key: uid(), label: '', length_um: toUm(system === 'metric' ? 6000 : 16, lenUnit), available: null }])
            }
          >
            + Ajouter une longueur
          </button>
        </div>
      </div>

      {/* Liste de coupe */}
      <div className="card">
        <h2>Liste de coupe <SectionHelp topic="cutlist" /></h2>
        <div className="toolbar" style={{ marginBottom: 12 }}>
          <button className="secondary" onClick={() => fileRef.current?.click()}>
            <Icon name="upload" /> Importer Excel / CSV
          </button>
          <button className="secondary" onClick={() => api.downloadTemplate()}>
            <Icon name="download" /> Modele d'import (CSV)
          </button>
          <input
            ref={fileRef}
            type="file"
            accept=".xlsx,.xls,.csv"
            style={{ display: 'none' }}
            onChange={(e) => {
              const f = e.target.files?.[0]
              if (f) doImport(f)
              e.target.value = ''
            }}
          />
        </div>
        <table>
          <thead>
            <tr>
              <th>partID</th>
              <th>model</th>
              <th>longueur</th>
              <th>quantite</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {parts.map((p) => (
              <tr key={p.key}>
                <td>
                  <input value={p.part_id} onChange={(e) => updPart(p.key, { part_id: e.target.value })} />
                </td>
                <td>
                  <input value={p.model} onChange={(e) => updPart(p.key, { model: e.target.value })} />
                </td>
                <td>
                  <LengthInput um={p.length_um} unit={lenUnit} onChange={(v) => updPart(p.key, { length_um: v })} />
                </td>
                <td className="num">
                  <input
                    type="number"
                    min={1}
                    value={p.qty}
                    onChange={(e) => updPart(p.key, { qty: Math.max(1, parseInt(e.target.value) || 1) })}
                  />
                </td>
                <td className="act">
                  <button className="ghost" onClick={() => setParts((r) => r.filter((x) => x.key !== p.key))} aria-label="Retirer">
                    <Icon name="close" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="toolbar" style={{ marginTop: 10 }}>
          <button
            className="secondary"
            onClick={() => setParts((r) => [...r, { key: uid(), part_id: '', model: '', length_um: toUm(system === 'metric' ? 1000 : 4, lenUnit), qty: 1 }])}
          >
            + Ajouter une piece
          </button>
        </div>
      </div>

      {/* Paramètres de perçage */}
      <div className="card">
        <h2>Perçage — répartition des trous <SectionHelp topic="drilling" /></h2>
        <p className="hint">
          Appliqué à toutes les pièces ; le nombre de trous s'ajuste selon la longueur (réparti
          symétriquement, marges de bout = points de départ).
        </p>
        <div className="row">
          <div style={{ flex: '0 1 170px' }}>
            <label>Entraxe (centre à centre)</label>
            <LengthInput um={pasUm} unit={fineUnit} onChange={setPasUm} />
          </div>
          <div style={{ flex: '0 1 160px' }}>
            <label>Marge de bout min</label>
            <LengthInput um={margeMinUm} unit={fineUnit} onChange={setMargeMinUm} />
          </div>
          <div style={{ flex: '0 1 160px' }}>
            <label>Marge de bout max</label>
            <LengthInput um={margeMaxUm} unit={fineUnit} onChange={setMargeMaxUm} />
          </div>
        </div>
      </div>

      <div className="card" style={{ textAlign: 'center' }}>
        <button className="big" onClick={runOptimize} disabled={loading}>
          {loading ? (
            'Calcul en cours…'
          ) : (
            <>
              <Icon name="tune" /> Optimiser (coupe + perçage)
            </>
          )}
        </button>
      </div>

      {solution && (
        <ResultView
          solution={solution}
          parts={buildProblem()?.apiParts ?? []}
          system={system}
          jobNumber={jobNumber}
          onExport={doExport}
          onSave={doSave}
          saving={saving}
          dbAvailable={dbAvailable}
        />
      )}

      {drillResults && drillResults.length > 0 && (
        <DrillingView results={drillResults} system={system} onExport={doExportDrilling} />
      )}

      <History jobs={jobs} dbAvailable={dbAvailable} onLoad={loadJob} onDelete={removeJob} onRefresh={refreshJobs} />
    </div>
  )
}
