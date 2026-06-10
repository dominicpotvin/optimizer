// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
import { useState } from 'react'
import Icon from './Icon'
import Modal from './Modal'
import { HELP } from './help'

export default function SectionHelp({ topic }: { topic: keyof typeof HELP }) {
  const [open, setOpen] = useState(false)
  const t = HELP[topic]
  return (
    <>
      <button
        type="button"
        className="help-dot"
        onClick={() => setOpen(true)}
        aria-label={`Aide : ${t.title}`}
        title="Comment ça marche ?"
      >
        <Icon name="help" />
      </button>
      {open && (
        <Modal title={t.title} onClose={() => setOpen(false)}>
          {t.body}
        </Modal>
      )}
    </>
  )
}
