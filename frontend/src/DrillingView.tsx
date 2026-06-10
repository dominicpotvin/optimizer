// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
import { formatLength, type UnitSystem } from './units'
import type { PartDrilling } from './types'
import Icon from './Icon'
import SectionHelp from './SectionHelp'

function Cote({ leftPct, widthPct, label }: { leftPct: number; widthPct: number; label: string }) {
  return (
    <div className="cote" style={{ left: `${leftPct}%`, width: `${widthPct}%` }}>
      <span className="cote-val">{label}</span>
    </div>
  )
}

function Piece({ d, system }: { d: PartDrilling; system: UnitSystem }) {
  const r = d.result
  const title = `${d.part_id || '(piece)'}${d.model ? ' · ' + d.model : ''} · ${formatLength(
    d.length_um,
    system,
  )}${d.qty > 1 ? ` · ×${d.qty}` : ''}`

  if (!r.ok) {
    return (
      <div className="drill-piece">
        <div className="drill-head">{title}</div>
        <div className="banner warn" style={{ margin: '6px 0 0' }}>
          <Icon name="warning" /> Perçage impossible pour cette longueur avec les paramètres actuels.
        </div>
      </div>
    )
  }

  return (
    <div className="drill-piece">
      <div className="drill-head">
        {title} — <strong>{r.n} trous</strong>, marge de bout{' '}
        <strong>{formatLength(r.marge_um, system)}</strong>{' '}
        <span className="muted">({r.parity})</span>
      </div>

      <div className="drill-bar" title={`${r.n} trous · entraxe ${formatLength(r.pas_um, system)}`}>
        {r.holes.map((h) => (
          <span
            key={h.index}
            className={`hole${h.is_center ? ' center' : ''}`}
            style={{ left: `${(h.from_a_um / d.length_um) * 100}%` }}
            title={`Trou ${h.index} — A: ${formatLength(h.from_a_um, system)} · B: ${formatLength(
              h.from_b_um,
              system,
            )}`}
          />
        ))}
      </div>

      {r.holes.length >= 1 && (
        <div className="drill-cotes">
          {/* début : bord A -> 1er trou (marge de bout) */}
          <Cote
            leftPct={0}
            widthPct={(r.holes[0].from_a_um / d.length_um) * 100}
            label={formatLength(r.holes[0].from_a_um, system)}
          />
          {/* 1er incrément : entraxe entre trou 1 et trou 2 */}
          {r.holes.length >= 2 && (
            <Cote
              leftPct={(r.holes[0].from_a_um / d.length_um) * 100}
              widthPct={((r.holes[1].from_a_um - r.holes[0].from_a_um) / d.length_um) * 100}
              label={formatLength(r.holes[1].from_a_um - r.holes[0].from_a_um, system)}
            />
          )}
          {/* fin : dernier trou -> bord B */}
          <Cote
            leftPct={(r.holes[r.holes.length - 1].from_a_um / d.length_um) * 100}
            widthPct={(r.holes[r.holes.length - 1].from_b_um / d.length_um) * 100}
            label={formatLength(r.holes[r.holes.length - 1].from_b_um, system)}
          />
        </div>
      )}

      <details className="drill-details">
        <summary>Positions des {r.n} trous (depuis A / depuis B)</summary>
        <table>
          <thead>
            <tr>
              <th>Trou</th>
              <th>Depuis A</th>
              <th>Depuis B</th>
            </tr>
          </thead>
          <tbody>
            {r.holes.map((h) => (
              <tr key={h.index} className={h.is_center ? 'center-row' : ''}>
                <td>
                  {h.index}
                  {h.is_center ? ' *' : ''}
                </td>
                <td>{formatLength(h.from_a_um, system)}</td>
                <td>{formatLength(h.from_b_um, system)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </div>
  )
}

interface Props {
  results: PartDrilling[]
  system: UnitSystem
  onExport: () => void
}

export default function DrillingView({ results, system, onExport }: Props) {
  return (
    <div className="card">
      <h2>Plan de perçage — points de départ par pièce <SectionHelp topic="drillplan" /></h2>
      <p className="hint">
        Répartition symétrique ; * = trou central. La « marge de bout » est le point de départ
        depuis chaque extrémité.
      </p>
      {results.map((d, i) => (
        <Piece key={i} d={d} system={system} />
      ))}
      <div className="toolbar" style={{ marginTop: 14 }}>
        <button onClick={onExport}>
          <Icon name="download" /> Exporter le perçage (CSV)
        </button>
      </div>
    </div>
  )
}
