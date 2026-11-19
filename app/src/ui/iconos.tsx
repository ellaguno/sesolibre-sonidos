// Íconos propios, de trazo, en línea: sin fuentes ni paquetes.
type P = { className?: string };
const base = { fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round', strokeLinejoin: 'round' } as const;

export const Tocar = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <path d="M8 5.5v13l11-6.5z" fill="currentColor" />
  </svg>
);
export const Pausa = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <path d="M8 5v14M16 5v14" {...base} strokeWidth={3} />
  </svg>
);
export const Alto = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <rect x="6.5" y="6.5" width="11" height="11" rx="2" fill="currentColor" />
  </svg>
);
export const Cerrar = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <path d="M6 6l12 12M18 6L6 18" {...base} />
  </svg>
);
export const Auriculares = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <path d="M4 14v-2a8 8 0 0116 0v2" {...base} />
    <path d="M4 14h3v6H6a2 2 0 01-2-2zM20 14h-3v6h1a2 2 0 002-2z" {...base} />
  </svg>
);
export const Luna = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <path d="M20 14.5A8 8 0 019.5 4a8 8 0 1010.5 10.5z" {...base} />
  </svg>
);
export const Reloj = ({ className }: P) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    <circle cx="12" cy="12" r="8.5" {...base} />
    <path d="M12 7.5V12l3 2" {...base} />
  </svg>
);
/** La marca de cada sala: ✔ comprobado · ◐ prometedor · ◌ exploración. */
export const MarcaSala = ({ sala, className }: P & { sala: 'comprobado' | 'prometedor' | 'exploracion' }) => (
  <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
    {sala === 'comprobado' && (
      <>
        <circle cx="12" cy="12" r="9" {...base} />
        <path d="M8 12.5l2.8 2.8L16 9.5" {...base} />
      </>
    )}
    {sala === 'prometedor' && (
      <>
        <circle cx="12" cy="12" r="9" {...base} />
        <path d="M12 3a9 9 0 000 18z" fill="currentColor" />
      </>
    )}
    {sala === 'exploracion' && <circle cx="12" cy="12" r="9" {...base} strokeDasharray="3.2 3.6" />}
  </svg>
);
