import type { ReactNode } from 'react';

import { useT } from '../../lib/i18n/I18nContext';
import { useMemoryEngine } from './useMemoryEngineCapabilities';

interface MemoryFamilyGateProps {
  /** Capability family the wrapped content needs (e.g. `tree`, `goals`). */
  family: string;
  children: ReactNode;
}

/**
 * Renders `children` when the bound memory engine has `family`, otherwise a
 * friendly "Not available with <engine>" state in place of an error. While the
 * engine is still loading it shows a skeleton instead of the children (so a
 * feature the engine lacks never flashes an error first). When the core cannot
 * answer at all it fails open. Memory paused on the `null` driver (a failed
 * engine bind) gates every family.
 */
export default function MemoryFamilyGate({ family, children }: MemoryFamilyGateProps) {
  const { t } = useT();
  const { loading, id, label, capabilities, current, error } = useMemoryEngine();

  if (loading) {
    return (
      <div
        data-testid="memory-family-loading"
        aria-busy="true"
        className="animate-pulse space-y-2 rounded-lg border border-line px-4 py-6">
        <div className="h-3 w-1/3 rounded bg-line" />
        <div className="h-3 w-2/3 rounded bg-line" />
      </div>
    );
  }

  if (id === 'null') {
    return (
      <div
        data-testid="memory-family-unavailable"
        data-family={family}
        className="rounded-lg border border-line px-4 py-6 text-center">
        <p className="text-sm font-medium text-content">{t('memoryEngine.paused')}</p>
        {current?.last_error ? (
          <p className="mt-1 text-xs text-content-muted">{current.last_error}</p>
        ) : null}
        <p className="mt-1 text-xs text-content-muted">{t('memoryEngine.unavailableHint')}</p>
      </div>
    );
  }

  if (error !== null || !id || !capabilities || capabilities.has(family)) {
    return <>{children}</>;
  }

  const engine = t(`memoryEngine.engine.${id}.label`, label ?? id);
  return (
    <div
      data-testid="memory-family-unavailable"
      data-family={family}
      className="rounded-lg border border-line px-4 py-6 text-center">
      <p className="text-sm font-medium text-content">
        {t('memoryEngine.unavailable').replace('{engine}', engine)}
      </p>
      <p className="mt-1 text-xs text-content-muted">{t('memoryEngine.unavailableHint')}</p>
    </div>
  );
}
