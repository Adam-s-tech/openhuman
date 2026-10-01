/**
 * Memory engine settings: pick exactly one active memory engine.
 *
 * The engine list comes from `openhuman.memory_engines_list`; the panel only
 * carries fallback labels keyed by engine id (i18n). Switching either copies
 * the existing memories first (`memory_engine_migrate`, polled) or just flips
 * the driver (`memory_engine_set`).
 *
 * debug logging: DEBUG=settings:memory-engine
 */
import debug from 'debug';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import { useCoreState } from '../../../providers/CoreStateProvider';
import { trackAnalyticsEvent } from '../../../services/analytics';
import { userScopedStorage } from '../../../store/userScopedStorage';
import { isLocalSessionToken } from '../../../utils/localSession';
import {
  type MemoryEngineDescriptor,
  memoryEngineMigrate,
  memoryEngineMigrateCancel,
  memoryEngineMigrateStatus,
  type MemoryEngineMigrateStatus,
  type MemoryEngineMigrateStep,
  memoryEngineSet,
  type MemoryEngineState,
  type MemoryEngineTarget,
} from '../../../utils/tauriCommands/memoryEngine';
import {
  invalidateMemoryEngine,
  useMemoryEngine,
} from '../../intelligence/useMemoryEngineCapabilities';
import { Alert, AlertDescription, Button, CenteredLoadingState } from '../../ui';
import { RadioGroupRoot } from '../../ui/RadioGroup';
import SettingsPanel from '../layout/SettingsPanel';
import MemoryEngineErrorAlert from './MemoryEngineErrorAlert';
import MemoryEngineOption, { type EngineFormValues } from './MemoryEngineOption';
import MemoryEngineSwitchDialog from './MemoryEngineSwitchDialog';
import {
  classifyMemoryEngineError,
  humanizeCapability,
  type MemoryEngineErrorKind,
  missingCapabilities,
} from './memoryEngineUtils';

const log = debug('settings:memory-engine');

/**
 * What a finished copy could not carry: the steps it skipped because an engine
 * does not serve them, and the content the new engine refused. Content the
 * user chose not to re-send is not left behind: they asked for that.
 */
export interface CopyCompletion {
  leftBehind: MemoryEngineMigrateStep[];
  refused: number;
}

export function copyCompletion(
  status: MemoryEngineMigrateStatus,
  replayContent: boolean
): CopyCompletion {
  const steps = status.steps ?? [];
  return {
    leftBehind: steps
      .filter(step => step.skipped_because !== null)
      .filter(step => replayContent || step.step !== 'content')
      .map(step => step.step),
    // Only re-sent content may fail without holding the switch back, so on a
    // finished job this is the content the new engine refused.
    refused: steps.reduce((sum, step) => sum + step.failed, 0),
  };
}

/** How often a running migration is polled. */
export const MIGRATE_POLL_INTERVAL_MS = 1000;

/** userScopedStorage key of the running migration `{ jobId, driver, replayContent }`. */
export const MIGRATE_JOB_STORAGE_KEY = 'memoryEngine.migrateJob';

interface StoredJob {
  jobId: string;
  driver: string;
  /** The content re-send choice the job was started with. */
  replayContent: boolean;
}

async function storeJob(job: StoredJob): Promise<void> {
  await userScopedStorage.setItem(MIGRATE_JOB_STORAGE_KEY, JSON.stringify(job));
}

async function clearStoredJob(): Promise<void> {
  await userScopedStorage.removeItem(MIGRATE_JOB_STORAGE_KEY);
}

async function readStoredJob(): Promise<StoredJob | null> {
  try {
    const raw = await userScopedStorage.getItem(MIGRATE_JOB_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as {
      jobId?: unknown;
      driver?: unknown;
      replayContent?: unknown;
    };
    return typeof parsed.jobId === 'string' && typeof parsed.driver === 'string'
      ? {
          jobId: parsed.jobId,
          driver: parsed.driver,
          // A job stored before the choice existed re-sent content.
          replayContent: parsed.replayContent !== false,
        }
      : null;
  } catch {
    return null;
  }
}

interface PanelError {
  kind: MemoryEngineErrorKind;
  message: string;
}

function toPanelError(err: unknown): PanelError {
  return {
    kind: classifyMemoryEngineError(err),
    message: err instanceof Error ? err.message : String(err ?? ''),
  };
}

function formFor(
  engine: MemoryEngineDescriptor,
  current: MemoryEngineState | null
): EngineFormValues {
  const same = current?.driver === engine.id;
  return {
    endpoint: (same ? current?.endpoint : null) ?? engine.default_endpoint ?? '',
    deployment: (same ? current?.deployment : null) ?? engine.deployments[0] ?? '',
    apiKey: '',
  };
}

export default function MemoryEnginePanel() {
  const { t } = useT();
  const { snapshot } = useCoreState();
  const signedIn = snapshot.auth.isAuthenticated && !isLocalSessionToken(snapshot.sessionToken);

  // One shared engines_list + engine_get for this view (and the Brain row / gates).
  const { engines, current, loading, error: loadError, refresh } = useMemoryEngine();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [forms, setForms] = useState<Record<string, EngineFormValues>>({});
  const [error, setError] = useState<PanelError | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [migration, setMigration] = useState<MemoryEngineMigrateStatus | null>(null);
  // Re-send synced content with the copy (the content step); on unless the user opts out.
  const [replayContent, setReplayContent] = useState(true);
  // What a finished copy could not carry, shown once the switch is done.
  const [completion, setCompletion] = useState<CopyCompletion | null>(null);
  // The replay choice the running job was started with (polling outlives the toggle).
  const replayRequestedRef = useRef(true);
  const mounted = useRef(true);
  // Driver the running migration is copying into (finishSwitch needs it after polling).
  const targetDriverRef = useRef<string>('');

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const load = useCallback(async () => {
    await refresh();
  }, [refresh]);

  // Always show fresh data on entry, then pick the active engine once known.
  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (current) setSelectedId(prev => prev ?? current.driver);
  }, [current]);

  // Resume a migration that was running when the panel was left.
  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const stored = await readStoredJob();
      if (!stored || cancelled) return;
      try {
        const status = await memoryEngineMigrateStatus(stored.jobId);
        if (cancelled || !mounted.current) return;
        if (status.state === 'running') {
          log(
            'resuming migration job=%s driver=%s replay_content=%s',
            stored.jobId,
            stored.driver,
            stored.replayContent
          );
          targetDriverRef.current = stored.driver;
          replayRequestedRef.current = stored.replayContent;
          setReplayContent(stored.replayContent);
          setSelectedId(stored.driver);
          setMigration(status);
          setSaving(true);
          setConfirming(true);
          setJobId(stored.jobId);
          return;
        }
        await clearStoredJob();
        if (status.state === 'done') await load();
      } catch {
        await clearStoredJob();
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [load]);

  const activeId = current?.driver ?? null;
  const activeEngine = engines.find(e => e.id === activeId);
  const selected = engines.find(e => e.id === selectedId);
  const form = selected ? (forms[selected.id] ?? formFor(selected, current)) : null;
  const isSameDriver = selected?.id === activeId;
  const keySaved = Boolean(isSameDriver && current?.has_credential);

  const patchForm = (patch: Partial<EngineFormValues>) => {
    if (!selected || !form) return;
    setForms(prev => ({ ...prev, [selected.id]: { ...form, ...patch } }));
  };

  const hostedBlocked = Boolean(selected?.hosted && !signedIn);
  const missingRequired = useMemo(() => {
    if (!selected || !form) return false;
    if (selected.needs_endpoint && !form.endpoint.trim()) return true;
    if (selected.needs_key && !selected.key_optional && !form.apiKey.trim() && !keySaved) {
      return true;
    }
    return false;
  }, [selected, form, keySaved]);

  const dirty = useMemo(() => {
    if (!isSameDriver || !form || !current) return false;
    const endpointChanged =
      selected?.needs_endpoint === true && form.endpoint !== (current.endpoint ?? '');
    const deploymentChanged =
      (selected?.deployments.length ?? 0) > 0 && form.deployment !== (current.deployment ?? '');
    return form.apiKey.trim() !== '' || endpointChanged || deploymentChanged;
  }, [isSameDriver, form, current, selected]);

  const buildTarget = (): MemoryEngineTarget | null => {
    if (!selected || !form) return null;
    const target: MemoryEngineTarget = { driver: selected.id };
    if (selected.needs_endpoint && form.endpoint.trim()) target.endpoint = form.endpoint.trim();
    if (selected.deployments.length > 0 && form.deployment) target.deployment = form.deployment;
    if ((selected.needs_key || selected.key_optional) && form.apiKey.trim()) {
      target.api_key = form.apiKey.trim();
    }
    return target;
  };

  const finishSwitch = useCallback(
    async (driver: string) => {
      trackAnalyticsEvent('memory_engine_switched', { engine: driver });
      setConfirming(false);
      setJobId(null);
      setMigration(null);
      void clearStoredJob();
      setForms({});
      setSelectedId(driver);
      await load();
    },
    [load]
  );

  const doSet = async () => {
    const target = buildTarget();
    if (!target) return;
    setSaving(true);
    setError(null);
    try {
      await memoryEngineSet(target);
      log('engine set driver=%s', target.driver);
      invalidateMemoryEngine();
      await refresh();
      if (!mounted.current) return;
      await finishSwitch(target.driver);
    } catch (err) {
      log('engine set failed: %o', err);
      if (mounted.current) {
        setConfirming(false);
        setError(toPanelError(err));
      }
    } finally {
      if (mounted.current) setSaving(false);
    }
  };

  const doMigrate = async () => {
    const target = buildTarget();
    if (!target) return;
    setSaving(true);
    setError(null);
    try {
      targetDriverRef.current = target.driver;
      setCompletion(null);
      replayRequestedRef.current = replayContent;
      const { job_id } = await memoryEngineMigrate(target, { replayContent });
      log(
        'migrate started job=%s driver=%s replay_content=%s',
        job_id,
        target.driver,
        replayContent
      );
      if (!mounted.current) return;
      setMigration({ state: 'running', copied: 0, total: null, error: null });
      setJobId(job_id);
      void storeJob({ jobId: job_id, driver: target.driver, replayContent });
    } catch (err) {
      log('migrate start failed: %o', err);
      if (mounted.current) {
        setConfirming(false);
        setError(toPanelError(err));
        setSaving(false);
      }
    }
  };

  // Poll the migration job until it settles.
  useEffect(() => {
    if (!jobId) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const tick = async () => {
      try {
        const status = await memoryEngineMigrateStatus(jobId);
        if (cancelled) return;
        setMigration(status);
        if (status.state === 'running') {
          timer = setTimeout(() => void tick(), MIGRATE_POLL_INTERVAL_MS);
          return;
        }
        setSaving(false);
        if (status.state === 'done') {
          log('migrate done job=%s copied=%d', jobId, status.copied);
          invalidateMemoryEngine();
          await finishSwitch(targetDriverRef.current);
          if (mounted.current) setCompletion(copyCompletion(status, replayRequestedRef.current));
        } else if (status.state === 'cancelled') {
          log('migrate cancelled job=%s', jobId);
          void clearStoredJob();
          setJobId(null);
          setConfirming(false);
          setMigration(null);
        } else {
          log('migrate failed job=%s', jobId);
          void clearStoredJob();
          setJobId(null);
          setConfirming(false);
          setMigration(null);
          setError(toPanelError(status.error ?? 'migration failed'));
        }
      } catch (err) {
        if (cancelled) return;
        log('migrate status failed: %o', err);
        void clearStoredJob();
        setSaving(false);
        setJobId(null);
        setConfirming(false);
        setMigration(null);
        setError(toPanelError(err));
      }
    };
    timer = setTimeout(() => void tick(), MIGRATE_POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
  }, [jobId, finishSwitch]);

  const doCancelMigration = async () => {
    if (!jobId) return;
    try {
      await memoryEngineMigrateCancel(jobId);
      log('cancel requested job=%s', jobId);
    } catch (err) {
      log('cancel failed: %o', err);
      if (mounted.current) setError(toPanelError(err));
    }
  };

  const onSwitchClick = () => {
    if (isSameDriver) {
      void doSet();
    } else {
      setConfirming(true);
    }
  };

  const lacking = missingCapabilities(activeEngine, selected).map(humanizeCapability);
  const switchDisabled =
    saving || !selected || hostedBlocked || missingRequired || (isSameDriver && !dirty);

  return (
    <SettingsPanel description={t('memoryEngine.description')}>
      <div className="space-y-4" data-testid="memory-engine-panel">
        {current?.fell_back_from || current?.last_error ? (
          <Alert variant="warning" data-testid="memory-engine-fallback">
            <AlertDescription>
              {current.fell_back_from
                ? t('memoryEngine.fallback')
                    .replace('{engine}', current.fell_back_from)
                    .replace('{reason}', current.last_error ?? '')
                : t('memoryEngine.lastError')}
            </AlertDescription>
          </Alert>
        ) : null}

        {completion && (completion.leftBehind.length > 0 || completion.refused > 0) ? (
          <Alert variant="info" data-testid="memory-engine-left-behind">
            <AlertDescription>
              {completion.leftBehind.length > 0 ? (
                <p>
                  {t('memoryEngine.notCopied').replace(
                    '{items}',
                    completion.leftBehind.map(step => t(`memoryEngine.step.${step}`)).join(', ')
                  )}
                </p>
              ) : null}
              {completion.refused > 0 ? (
                <p data-testid="memory-engine-content-refused">
                  {t('memoryEngine.contentRefused').replace('{count}', String(completion.refused))}
                </p>
              ) : null}
            </AlertDescription>
          </Alert>
        ) : null}

        {error ? (
          <MemoryEngineErrorAlert error={error.message} kind={error.kind} />
        ) : loadError ? (
          <MemoryEngineErrorAlert error={loadError} />
        ) : null}

        {loading ? (
          <CenteredLoadingState />
        ) : (
          <>
            <RadioGroupRoot
              aria-label={t('memoryEngine.title')}
              value={selectedId ?? ''}
              onValueChange={id => {
                setSelectedId(id);
                setError(null);
              }}>
              {engines.map(engine => (
                <MemoryEngineOption
                  key={engine.id}
                  engine={engine}
                  isActive={engine.id === activeId}
                  isSelected={engine.id === selectedId}
                  keySaved={engine.id === activeId && Boolean(current?.has_credential)}
                  disabledReason={engine.hosted && !signedIn ? 'signed_out' : null}
                  form={engine.id === selectedId && form ? form : formFor(engine, current)}
                  onFormChange={patchForm}
                />
              ))}
            </RadioGroupRoot>
            <div className="flex justify-end">
              <Button
                variant="primary"
                size="sm"
                analyticsId="memory-engine-switch"
                data-testid="memory-engine-switch"
                disabled={switchDisabled}
                onClick={onSwitchClick}>
                {isSameDriver ? t('memoryEngine.save') : t('memoryEngine.switch')}
              </Button>
            </div>
          </>
        )}

        {confirming && selected ? (
          <MemoryEngineSwitchDialog
            targetLabel={t(`memoryEngine.engine.${selected.id}.label`, selected.label)}
            lacking={lacking}
            migration={migration}
            busy={saving}
            hosted={Boolean(selected.hosted)}
            replayContent={replayContent}
            onReplayContentChange={setReplayContent}
            onCopy={() => void doMigrate()}
            onSkipCopy={() => void doSet()}
            onCancel={() => setConfirming(false)}
            onCancelMigration={() => void doCancelMigration()}
          />
        ) : null}
      </div>
    </SettingsPanel>
  );
}
