import { useT } from '../../../lib/i18n/I18nContext';
import type { MemoryEngineMigrateStatus } from '../../../utils/tauriCommands/memoryEngine';
import { Button, ModalShell, Progress } from '../../ui';

interface MemoryEngineSwitchDialogProps {
  targetLabel: string;
  /** Capabilities of the current engine that the target lacks (humanized). */
  lacking: string[];
  /** Set once "Copy & switch" started; swaps the choices for a progress view. */
  migration: MemoryEngineMigrateStatus | null;
  busy: boolean;
  errorMessage: string | null;
  onCopy: () => void;
  onSkipCopy: () => void;
  onCancel: () => void;
}

/** "Copy my existing memories?" confirmation, then migration progress. */
export default function MemoryEngineSwitchDialog({
  targetLabel,
  lacking,
  migration,
  busy,
  errorMessage,
  onCopy,
  onSkipCopy,
  onCancel,
}: MemoryEngineSwitchDialogProps) {
  const { t } = useT();
  const migrating = migration !== null && migration.state === 'running';
  const percent =
    migration && migration.total && migration.total > 0
      ? Math.min(100, Math.round((migration.copied / migration.total) * 100))
      : null;

  return (
    <ModalShell
      title={t('memoryEngine.dialog.title').replace('{engine}', targetLabel)}
      titleId="memory-engine-switch-title"
      testId="memory-engine-switch-dialog"
      onClose={onCancel}
      maxWidthClassName="max-w-md"
      closePolicy={busy ? { escape: false, backdrop: false, button: false } : undefined}
      footer={
        migrating ? null : (
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              variant="tertiary"
              size="sm"
              analyticsId="memory-engine-cancel"
              data-testid="memory-engine-cancel"
              disabled={busy}
              onClick={onCancel}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              analyticsId="memory-engine-switch-only"
              data-testid="memory-engine-switch-only"
              disabled={busy}
              onClick={onSkipCopy}>
              {t('memoryEngine.dialog.switchOnly')}
            </Button>
            <Button
              variant="primary"
              size="sm"
              analyticsId="memory-engine-copy-switch"
              data-testid="memory-engine-copy-switch"
              disabled={busy}
              onClick={onCopy}>
              {t('memoryEngine.dialog.copySwitch')}
            </Button>
          </div>
        )
      }>
      <div className="space-y-3 text-sm text-content-secondary">
        {migrating ? (
          <div data-testid="memory-engine-progress" className="space-y-2">
            <p>{t('memoryEngine.dialog.copying')}</p>
            <Progress value={percent} aria-label={t('memoryEngine.dialog.copying')} />
            <p className="text-xs text-content-muted">
              {migration.total !== null
                ? t('memoryEngine.dialog.progress')
                    .replace('{copied}', String(migration.copied))
                    .replace('{total}', String(migration.total))
                : t('memoryEngine.dialog.progressUnknown').replace(
                    '{copied}',
                    String(migration.copied)
                  )}
            </p>
          </div>
        ) : (
          <>
            <p>{t('memoryEngine.dialog.body')}</p>
            {lacking.length > 0 ? (
              <div data-testid="memory-engine-lacking">
                <p className="font-medium text-content">{t('memoryEngine.dialog.lacking')}</p>
                <ul className="mt-1 list-disc pl-5 text-xs text-content-muted">
                  {lacking.map(cap => (
                    <li key={cap}>{cap}</li>
                  ))}
                </ul>
              </div>
            ) : null}
          </>
        )}
        {errorMessage ? (
          <p role="alert" className="text-xs text-coral-600">
            {errorMessage}
          </p>
        ) : null}
      </div>
    </ModalShell>
  );
}
