import { Navigate, useLocation } from 'react-router-dom';

/**
 * Back-compat redirect for the retired top-level `/brain` page, which now lives
 * at Connections → Integrations → Brain (`/connections?tab=brain`).
 *
 * Brain used to own `?tab=` (graph|goals|sources|sync|welcome) and `?view=`.
 * Connections owns `?tab=` now, so the old sub-tab moves to `?brain=`; every
 * other param (e.g. `view=history`) and the hash are carried over unchanged.
 */
export default function BrainRedirect() {
  const { search, hash } = useLocation();
  const params = new URLSearchParams(search);
  const legacyTab = params.get('tab');
  params.set('tab', 'brain');
  if (legacyTab) params.set('brain', legacyTab);
  return <Navigate to={`/connections?${params.toString()}${hash}`} replace />;
}
