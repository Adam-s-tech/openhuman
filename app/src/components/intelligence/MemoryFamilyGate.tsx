import type { ReactNode } from 'react';

import { useT } from '../../lib/i18n/I18nContext';
import { useMemoryEngineCapabilities } from './useMemoryEngineCapabilities';

interface MemoryFamilyGateProps {
  /** Capability family the wrapped content needs (e.g. `tree`, `goals`). */
  family: string;
  children: ReactNode;
}

/**
 * Renders `children` when the bound memory engine has `family` (or when that is
 * not yet known), otherwise a friendly "Not available with <engine>" state in
 * place of an error.
 */
export default function MemoryFamilyGate({ family, children }: MemoryFamilyGateProps) {
  const { t } = useT();
  const { id, label, capabilities } = useMemoryEngineCapabilities();

  if (!id || !capabilities || capabilities.has(family)) return <>{children}</>;

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
