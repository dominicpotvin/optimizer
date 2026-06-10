import { formatLength, type UnitSystem } from './units'
import type { BarPlan, PartType, Solution } from './types'

function colorFor(label: string): string {
  let h = 0
  for (let i = 0; i < label.length; i++) h = (h * 31 + label.charCodeAt(i)) >>> 0
  const hue = h % 360
  return `hsl(${hue} 62% 52%)`
}

function Bar({ bar, maxStock, system }: { bar: BarPlan; maxStock: number; system: UnitSystem }) {
  const widthPct = (bar.stock_length_um / maxStock) * 100
  const kerfEach = bar.cuts.length > 0 ? bar.kerf_total_um / bar.cuts.length : 0
  let x = 0
  const segs = bar.cuts.map((cut, i) => {
    const seg = (
      <g key={i}>
        <rect
          x={x}
          y={0}
          width={cut.length_um}
          height={100}
          fill={colorFor(cut.label)}
          stroke="rgba(0,0,0,0.35)"
          strokeWidth={300}
        >
          <title>
            {cut.label} — {formatLength(cut.length_um, system)}
          </title>
        </rect>
      </g>
    )
    x += cut.length_um + kerfEach
    return seg
  })
  const offcutColor = bar.reusable ? 'var(--reuse)' : 'var(--warn)'

  return (
    <div className="bar-row">
      <div className="lab">{bar.stock_label}</div>
      <div className="svgwrap" style={{ width: `${widthPct}%` }}>
        <svg
          viewBox={`0 0 ${bar.stock_length_um} 100`}
          preserveAspectRatio="none"
          style={{ width: '100%', height: 34, display: 'block', borderRadius: 4, background: 'var(--surface-2)' }}
        >
          {segs}
          {bar.offcut_um > 0 && (
            <rect x={x} y={0} width={bar.stock_length_um - x} height={100} fill={offcutColor} opacity={0.4}>
              <title>Chute — {formatLength(bar.offcut_um, system)}{bar.reusable ? ' (reutilisable)' : ''}</title>
            </rect>
          )}
        </svg>
      </div>
      <div className="meta">
        chute{' '}
        <span className={bar.reusable ? 'reuse' : 'waste'}>{formatLength(bar.offcut_um, system)}</span>
      </div>
    </div>
  )
}

interface Props {
  solution: Solution
  parts: PartType[]
  system: UnitSystem
  jobNumber: string
  onExport: () => void
  onSave: () => void
  saving: boolean
  dbAvailable: boolean
}

export default function ResultView({
  solution,
  parts,
  system,
  jobNumber,
  onExport,
  onSave,
  saving,
  dbAvailable,
}: Props) {
  const s = solution.summary
  const maxStock = Math.max(1, ...solution.bars.map((b) => b.stock_length_um))

  const distinct = new Map<string, string>()
  for (const b of solution.bars) for (const c of b.cuts) distinct.set(c.label, colorFor(c.label))

  return (
    <div className="card">
      <h2>Resultat — plan de coupe</h2>

      {!solution.complete && (
        <div className="banner warn">
          ⚠ {solution.unplaced.length} piece(s) n'ont pas pu etre placees (stock insuffisant ou
          piece plus longue que toute barre disponible).
        </div>
      )}

      <div className="stats">
        <div className="stat">
          <div className="k">Barres necessaires</div>
          <div className="v">{s.total_bars}</div>
        </div>
        <div className="stat">
          <div className="k">Utilisation matiere</div>
          <div className="v">{s.utilization_pct.toFixed(1)} %</div>
        </div>
        <div className="stat">
          <div className="k">Perte reelle</div>
          <div className="v">{formatLength(s.real_waste_um, system)}</div>
        </div>
        <div className="stat">
          <div className="k">Chutes reutilisables</div>
          <div className="v">
            {s.reusable_count} <small>· {formatLength(s.reusable_offcut_um, system)}</small>
          </div>
        </div>
        <div className="stat">
          <div className="k">Trait de scie total</div>
          <div className="v">{formatLength(s.total_kerf_um, system)}</div>
        </div>
      </div>

      <div className="muted" style={{ marginBottom: 6 }}>
        Detail par longueur :{' '}
        {s.bars_by_stock.map((u) => `${u.count} × ${u.label}`).join('  ·  ') || '—'}
      </div>

      {distinct.size > 0 && (
        <div className="legend">
          {[...distinct.entries()].map(([label, color]) => (
            <span key={label}>
              <span className="sw" style={{ background: color }} />
              {label}
            </span>
          ))}
          <span>
            <span className="sw" style={{ background: 'var(--reuse)', opacity: 0.5 }} />
            chute reutilisable
          </span>
          <span>
            <span className="sw" style={{ background: 'var(--warn)', opacity: 0.5 }} />
            perte
          </span>
        </div>
      )}

      <div style={{ marginTop: 6 }}>
        {solution.bars.map((bar, i) => (
          <Bar key={i} bar={bar} maxStock={maxStock} system={system} />
        ))}
      </div>

      <div className="toolbar" style={{ marginTop: 16 }}>
        <button onClick={onExport}>⬇ Exporter le plan (CSV)</button>
        <button className="secondary" onClick={onSave} disabled={saving || !dbAvailable} title={!dbAvailable ? 'Base de donnees indisponible' : ''}>
          {saving ? 'Enregistrement…' : '💾 Enregistrer le job'}
        </button>
        {!dbAvailable && <span className="muted">Base de donnees non connectee — enregistrement desactive.</span>}
      </div>
    </div>
  )
}
