import { Navigate, useLocation } from 'react-router-dom';

/**
 * Back-compat redirect for the retired top-level `/brain` page, which now lives
 * at Connections → Integrations → Memory (`/connections?tab=brain`).
 *
 * Brain used to own `?tab=` (graph|goals|sources|sync|welcome) and `?view=`.
 * Connections owns `?tab=` now, so the old sub-tab moves to `?brain=`; every
 * other param (e.g. `view=history`) and the hash are carried over unchanged,
 * after the canonical `tab`/`brain` pair.
 */
export default function BrainRedirect() {
  const { search, hash } = useLocation();
  const legacy = new URLSearchParams(search);
  const params = new URLSearchParams({ tab: 'brain' });
  const legacyTab = legacy.get('tab');
  if (legacyTab) params.set('brain', legacyTab);
  legacy.forEach((value, key) => {
    if (key !== 'tab') params.append(key, value);
  });
  return <Navigate to={`/connections?${params.toString()}${hash}`} replace />;
}
