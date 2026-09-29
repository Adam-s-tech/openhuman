import debug from 'debug';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { useT } from '../../lib/i18n/I18nContext';
import { memoryEngineGet, memoryEnginesList } from '../../utils/tauriCommands/memoryEngine';
import { Button } from '../ui';

const log = debug('brain:memory-engine-row');

/**
 * "Memory engine: <label> · Change" — a one-line summary on the Brain page that
 * links to Settings → memory engine. Renders nothing until the core answers (or
 * if it does not support memory engines), so it never blocks the page.
 */
export default function MemoryEngineRow() {
  const { t } = useT();
  const navigate = useNavigate();
  const [engine, setEngine] = useState<{ id: string; label: string } | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [list, state] = await Promise.all([memoryEnginesList(), memoryEngineGet()]);
        if (cancelled) return;
        const label = list.engines.find(e => e.id === state.driver)?.label ?? state.driver;
        setEngine({ id: state.driver, label });
      } catch (err) {
        log('unavailable: %o', err);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

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
