/**
 * Merge Y icon with a small watching eye in the fork: Lynter looks for what is
 * out of place. Colors come from CSS variables so it follows the theme.
 */
export function Icon() {
  return (
    <svg viewBox="0 0 48 48" aria-hidden="true" focusable="false" fill="none" strokeLinecap="round" strokeWidth="6">
      <path className="arm" d="M12 8 L24 26" />
      <path className="arm" d="M36 8 L24 26" />
      <path className="stem" d="M24 26 V41" />
      <path className="eye" d="M19.8 12 Q24 7.6 28.2 12 Q24 16.4 19.8 12 Z" strokeWidth="0" />
      <ellipse className="pupil" cx="24" cy="12" rx="0.9" ry="2.6" strokeWidth="0" />
    </svg>
  );
}
