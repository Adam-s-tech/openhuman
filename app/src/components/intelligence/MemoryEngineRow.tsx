import { useNavigate } from 'react-router-dom';

import { useT } from '../../lib/i18n/I18nContext';
import { Button } from '../ui';
import { useMemoryEngine } from './useMemoryEngineCapabilities';

/**
 * "Memory engine: <label> · Change" — a one-line summary on the Brain page that
 * links to Settings → memory engine. Renders nothing until the core answers (or
 * if it does not support memory engines), so it never blocks the page.
 */
export default function MemoryEngineRow() {
  const { t } = useT();
  const navigate = useNavigate();
  const { id, label } = useMemoryEngine();
  const engine = id ? { id, label: label ?? id } : null;

  if (!engine) return null;
  return (
    <div
      data-testid="brain-memory-engine-row"
      className="flex items-center justify-between gap-3 rounded-lg border border-line px-4 py-2.5 text-sm">
      <span className="text-content-secondary">
        {t('memoryEngine.row.label')}{' '}
        <span className="font-medium text-content">
          {t(`memoryEngine.engine.${engine.id}.label`, engine.label)}
        </span>
      </span>
      <Button
        variant="tertiary"
        size="xs"
        analyticsId="brain-memory-engine-change"
        data-testid="brain-memory-engine-change"
        onClick={() => navigate('/settings/memory-engine')}>
        {t('memoryEngine.row.change')}
      </Button>
    </div>
  );
}
