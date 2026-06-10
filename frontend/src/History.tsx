// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
import type { JobSummary } from './types'
import Icon from './Icon'

interface Props {
  jobs: JobSummary[]
  dbAvailable: boolean
  onLoad: (id: string) => void
  onDelete: (id: string) => void
  onRefresh: () => void
}

export default function History({ jobs, dbAvailable, onLoad, onDelete, onRefresh }: Props) {
  return (
    <div className="card">
      <h2>
        Historique des jobs
        <button className="ghost" style={{ marginLeft: 'auto' }} onClick={onRefresh}>
          <Icon name="refresh" /> Rafraichir
        </button>
      </h2>
      {!dbAvailable && <div className="muted">Base de donnees non connectee.</div>}
      {dbAvailable && jobs.length === 0 && <div className="muted">Aucun job enregistre.</div>}
      {jobs.map((j) => (
        <div className="history-item" key={j.id}>
          <div>
            <span className="jn">{j.job_number || '(sans numero)'}</span>{' '}
            <span className="dt">
              · {new Date(j.created_at).toLocaleString('fr-CA')}
              {j.total_bars != null ? ` · ${j.total_bars} barres` : ''}
            </span>
          </div>
          <div className="toolbar">
            <button className="secondary" onClick={() => onLoad(j.id)}>
              Charger
            </button>
            <button className="danger" onClick={() => onDelete(j.id)}>
              Suppr.
            </button>
          </div>
        </div>
      ))}
    </div>
  )
}
