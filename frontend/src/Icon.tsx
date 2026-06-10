// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
// Icône Material Symbols (police chargée dans index.html). Aucune émoji dans l'app.
export default function Icon({ name, className }: { name: string; className?: string }) {
  return (
    <span className={`material-symbols-outlined${className ? ' ' + className : ''}`} aria-hidden="true">
      {name}
    </span>
  )
}
