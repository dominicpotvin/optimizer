// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
import { useEffect, useState } from 'react'
import { fmtValue, parseLength, type LengthUnit } from './units'

interface Props {
  um: number
  unit: LengthUnit
  onChange: (um: number) => void
  placeholder?: string
}

/**
 * Champ de longueur : saisie texte (accepte fractions « 3-1/2 », « 1/8 »)
 * dans l'unite courante. Convertit en micrometres a la validation.
 */
export default function LengthInput({ um, unit, onChange, placeholder }: Props) {
  const [text, setText] = useState(fmtValue(um, unit))

  // Resynchronise quand la valeur externe ou l'unite change.
  useEffect(() => {
    setText(fmtValue(um, unit))
  }, [um, unit])

  const commit = (raw: string) => {
    const v = parseLength(raw, unit)
    if (v !== null && v >= 0) {
      onChange(v)
      setText(fmtValue(v, unit))
    } else {
      setText(fmtValue(um, unit)) // rollback
    }
  }

  return (
    <span className="lenfield">
      <input
        value={text}
        placeholder={placeholder}
        onChange={(e) => setText(e.target.value)}
        onBlur={(e) => commit(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            commit((e.target as HTMLInputElement).value)
            ;(e.target as HTMLInputElement).blur()
          }
        }}
        inputMode="decimal"
      />
      <span className="u">{unit}</span>
    </span>
  )
}
