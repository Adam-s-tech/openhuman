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
import { useNavigate } from 'react-router-dom';

import { useT } from '../../../lib/i18n/I18nContext';
import { useCoreState } from '../../../providers/CoreStateProvider';
import { trackAnalyticsEvent } from '../../../services/analytics';
import { isLocalSessionToken } from '../../../utils/localSession';
import {
  type MemoryEngineDescriptor,
  memoryEngineGet,
  memoryEngineMigrate,
  memoryEngineMigrateStatus,
  type MemoryEngineMigrateStatus,
  memoryEngineSet,
  memoryEnginesList,
  type MemoryEngineState,
  type MemoryEngineTarget,
} from '../../../utils/tauriCommands/memoryEngine';
import { Alert, AlertDescription, Button, CenteredLoadingState } from '../../ui';
import { RadioGroupRoot } from '../../ui/RadioGroup';
import SettingsPanel from '../layout/SettingsPanel';
import MemoryEngineOption, { type EngineFormValues } from './MemoryEngineOption';
import MemoryEngineSwitchDialog from './MemoryEngineSwitchDialog';
import {
  classifyMemoryEngineError,
  humanizeCapability,
  type MemoryEngineErrorKind,
  missingCapabilities,
} from './memoryEngineUtils';

const log = debug('settings:memory-engine');

/** How often a running migration is polled. */
export const MIGRATE_POLL_INTERVAL_MS = 1000;

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
  const navigate = useNavigate();
  const { snapshot } = useCoreState();
  const signedIn = snapshot.auth.isAuthenticated && !isLocalSessionToken(snapshot.sessionToken);

  const [engines, setEngines] = useState<MemoryEngineDescriptor[]>([]);
  const [current, setCurrent] = useState<MemoryEngineState | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [forms, setForms] = useState<Record<string, EngineFormValues>>({});
  const [error, setError] = useState<PanelError | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [migration, setMigration] = useState<MemoryEngineMigrateStatus | null>(null);
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
    try {
      const [list, state] = await Promise.all([memoryEnginesList(), memoryEngineGet()]);
      if (!mounted.current) return;
      log('loaded engines=%d active=%s', list.engines.length, state.driver);
      setEngines(list.engines);
      setCurrent(state);
      setSelectedId(prev => prev ?? state.driver);
      setError(null);
    } catch (err) {
      log('load failed: %o', err);
      if (mounted.current) setError(toPanelError(err));
    } finally {
      if (mounted.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
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
      const { job_id } = await memoryEngineMigrate(target);
      log('migrate started job=%s driver=%s', job_id, target.driver);
      if (!mounted.current) return;
      setMigration({ state: 'running', copied: 0, total: null, error: null });
      setJobId(job_id);
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
          await finishSwitch(targetDriverRef.current);
        } else {
          log('migrate failed job=%s', jobId);
          setJobId(null);
          setConfirming(false);
          setMigration(null);
          setError(toPanelError(status.error ?? 'migration failed'));
        }
      } catch (err) {
        if (cancelled) return;
        log('migrate status failed: %o', err);
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

  const onSwitchClick = () => {
    if (isSameDriver) {
      void doSet();
    } else {
      setConfirming(true);
    }
  };

  const errorText = (e: PanelError) =>
    e.kind === 'insufficient_credits'
      ? t('memoryEngine.error.insufficientCredits')
      : e.kind === 'session_expired'
        ? t('memoryEngine.error.sessionExpired')
        : e.kind === 'backend_unavailable'
          ? t('memoryEngine.error.backendUnavailable')
          : t('memoryEngine.error.generic');

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
                ? t('memoryEngine.fallback').replace('{engine}', current.fell_back_from)
                : t('memoryEngine.lastError')}
            </AlertDescription>
          </Alert>
        ) : null}

        {error ? (
          <Alert variant="destructive" data-testid={`memory-engine-error-${error.kind}`}>
            <AlertDescription>
              <span>{errorText(error)}</span>
              {error.kind === 'insufficient_credits' ? (
                <Button
                  variant="tertiary"
                  size="xs"
                  className="ml-2"
                  analyticsId="memory-engine-open-billing"
                  data-testid="memory-engine-open-billing"
                  onClick={() => navigate('/settings/account')}>
                  {t('memoryEngine.error.openBilling')}
                </Button>
              ) : null}
              {error.kind === 'session_expired' ? (
                <Button
                  variant="tertiary"
                  size="xs"
                  className="ml-2"
                  analyticsId="memory-engine-sign-in"
                  data-testid="memory-engine-sign-in"
                  onClick={() => navigate('/')}>
                  {t('memoryEngine.error.signIn')}
                </Button>
              ) : null}
            </AlertDescription>
          </Alert>
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
            onCopy={() => void doMigrate()}
            onSkipCopy={() => void doSet()}
            onCancel={() => setConfirming(false)}
          />
        ) : null}
      </div>
    </SettingsPanel>
  );
}
