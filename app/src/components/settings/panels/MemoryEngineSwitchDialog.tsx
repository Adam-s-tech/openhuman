import { useT } from '../../../lib/i18n/I18nContext';
import type { MemoryEngineMigrateStatus } from '../../../utils/tauriCommands/memoryEngine';
import { Button, Checkbox, ModalShell, Progress } from '../../ui';

interface MemoryEngineSwitchDialogProps {
  targetLabel: string;
  /** Capabilities of the current engine that the target lacks (humanized). */
  lacking: string[];
  /** Set once "Copy & switch" started; swaps the choices for a progress view. */
  migration: MemoryEngineMigrateStatus | null;
  busy: boolean;
  /** Whether the target bills for what it reads (TinyHumans memory). */
  hosted: boolean;
  /** Re-send synced content with the copy, so the new engine rebuilds its summaries. */
  replayContent: boolean;
  onReplayContentChange: (next: boolean) => void;
  onCopy: () => void;
  onSkipCopy: () => void;
  onCancel: () => void;
  /** Stops the running copy (the active engine is left unchanged). */
  onCancelMigration: () => void;
}

/** "Copy my existing memories?" confirmation, then migration progress. */
export default function MemoryEngineSwitchDialog({
  targetLabel,
  lacking,
  migration,
  busy,
  hosted,
  replayContent,
  onReplayContentChange,
  onCopy,
  onSkipCopy,
  onCancel,
  onCancelMigration,
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
        migrating ? (
          <div className="flex justify-end">
            <Button
              variant="tertiary"
              size="sm"
              analyticsId="memory-engine-cancel-migration"
              data-testid="memory-engine-cancel-migration"
              onClick={onCancelMigration}>
              {t('memoryEngine.dialog.cancelMigration')}
            </Button>
          </div>
        ) : (
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
            <p className="text-xs text-content-muted" data-testid="memory-engine-progress-line">
              {migration.step && migration.step !== 'records'
                ? t('memoryEngine.dialog.stepProgress')
                    .replace('{step}', t(`memoryEngine.step.${migration.step}`))
                    .replace('{count}', String(migration.step_read ?? 0))
                : migration.total !== null
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
            <p className="text-xs text-content-muted">{t('memoryEngine.dialog.copies')}</p>
            <label className="flex items-start gap-2" htmlFor="memory-engine-replay-content">
              <Checkbox
                id="memory-engine-replay-content"
                data-testid="memory-engine-replay-content"
                checked={replayContent}
                disabled={busy}
                onCheckedChange={onReplayContentChange}
                className="mt-0.5"
              />
              <span>
                <span className="text-content">{t('memoryEngine.dialog.replayContent')}</span>
                {hosted ? (
                  <span className="block text-xs text-content-muted">
                    {t('memoryEngine.dialog.replayContentHint')}
                  </span>
                ) : null}
              </span>
            </label>
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
      </div>
    </ModalShell>
  );
}
