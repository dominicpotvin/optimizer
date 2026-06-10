// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
export type UnitSystem = 'imperial' | 'metric'
export type LengthUnit = 'pi' | 'po' | 'mm'

export const UM_PER_MM = 1000
export const UM_PER_INCH = 25400
export const UM_PER_FOOT = 304800

export function toUm(value: number, unit: LengthUnit): number {
  switch (unit) {
    case 'pi':
      return Math.round(value * UM_PER_FOOT)
    case 'po':
      return Math.round(value * UM_PER_INCH)
    case 'mm':
      return Math.round(value * UM_PER_MM)
  }
}

export function fromUm(um: number, unit: LengthUnit): number {
  switch (unit) {
    case 'pi':
      return um / UM_PER_FOOT
    case 'po':
      return um / UM_PER_INCH
    case 'mm':
      return um / UM_PER_MM
  }
}

function trim(n: number, decimals: number): string {
  return parseFloat(n.toFixed(decimals)).toString()
}

/** Valeur formatee dans une unite simple (pour les champs de saisie). */
export function fmtValue(um: number, unit: LengthUnit): string {
  const d = unit === 'mm' ? 1 : 3
  return trim(fromUm(um, unit), d)
}

/** Affichage lisible : pi'-po" en imperial, mm en metrique. */
export function formatLength(um: number, system: UnitSystem): string {
  if (system === 'metric') {
    return `${trim(um / UM_PER_MM, 1)} mm`
  }
  const totalInches = um / UM_PER_INCH
  const sign = totalInches < 0 ? '-' : ''
  const ti = Math.abs(totalInches)
  const feet = Math.floor(ti / 12 + 1e-9)
  const inches = ti - feet * 12
  if (feet > 0) {
    const inStr = inches > 0.0005 ? ` ${trim(inches, 3)}"` : ''
    return `${sign}${feet}'${inStr}`
  }
  return `${sign}${trim(ti, 3)}"`
}

function parseFraction(s: string): number | null {
  const parts = s.split('/')
  if (parts.length !== 2) return null
  const n = parseFloat(parts[0].trim())
  const d = parseFloat(parts[1].trim())
  if (isNaN(n) || isNaN(d) || d === 0) return null
  return n / d
}

/** Nombre simple : "3", "3.5", "1/8", "3-1/2", "3 1/2". */
function parseNumber(raw: string): number | null {
  const s = raw.trim()
  if (!s) return null
  const norm = s.replace(/-/g, ' ')
  const parts = norm.split(/\s+/).filter(Boolean)
  if (parts.length === 2 && parts[1].includes('/')) {
    const w = parseFloat(parts[0])
    const f = parseFraction(parts[1])
    if (isNaN(w) || f === null) return null
    return w + f
  }
  if (parts.length === 1) {
    if (parts[0].includes('/')) return parseFraction(parts[0])
    const v = parseFloat(parts[0])
    return isNaN(v) ? null : v
  }
  return null
}

function indexOfAny(s: string, needles: string[]): number {
  let best = -1
  for (const n of needles) {
    const i = s.indexOf(n)
    if (i >= 0 && (best < 0 || i < best)) best = i
  }
  return best
}

/**
 * Analyse une longueur en texte -> micrometres. `def` = unite par defaut
 * quand aucun suffixe n'est present. Accepte fractions, nombres mixtes,
 * pi/po combines, mm.
 */
export function parseLength(raw: string, def: LengthUnit): number | null {
  const s = raw.trim().toLowerCase()
  if (!s) return null

  if (s.includes('mm') || /millim[eè]tres?/.test(s)) {
    const n = parseNumber(s.replace(/mm/g, ' ').replace(/millim[eè]tres?/g, ' '))
    return n === null ? null : toUm(n, 'mm')
  }

  const feetIdx = indexOfAny(s, ["'", 'pied', 'pi'])
  const inchPresent =
    s.includes('"') || s.includes('po') || s.includes('pouce') || s.includes('inch')

  if (feetIdx >= 0 || inchPresent) {
    let total = 0
    let parsedAny = false
    let rest = s

    if (feetIdx >= 0) {
      const feetPart = s.slice(0, feetIdx)
      const after = s.slice(feetIdx)
      let skip = 0
      if (after.startsWith("'")) skip = 1
      else if (after.startsWith('pied')) skip = 4
      else if (after.startsWith('pi')) skip = 2
      rest = after.slice(skip)
      const fv = parseNumber(feetPart)
      if (fv !== null) {
        total += toUm(fv, 'pi')
        parsedAny = true
      }
    }

    const inchStr = rest
      .replace(/"/g, '')
      .replace(/pouces?/g, '')
      .replace(/inches?|inch/g, '')
      .replace(/po/g, '')
      .trim()
    const iv = parseNumber(inchStr)
    if (iv !== null) {
      total += toUm(iv, 'po')
      parsedAny = true
    }
    return parsedAny ? total : null
  }

  const n = parseNumber(s)
  return n === null ? null : toUm(n, def)
}

/** Unite de saisie par defaut selon le systeme. */
export function lengthUnitFor(system: UnitSystem): LengthUnit {
  return system === 'metric' ? 'mm' : 'pi'
}
export function fineUnitFor(system: UnitSystem): LengthUnit {
  // kerf et petites mesures : pouces en imperial, mm en metrique
  return system === 'metric' ? 'mm' : 'po'
}
