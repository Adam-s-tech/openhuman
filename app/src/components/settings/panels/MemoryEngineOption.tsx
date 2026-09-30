import { useT } from '../../../lib/i18n/I18nContext';
import type { MemoryEngineDescriptor } from '../../../utils/tauriCommands/memoryEngine';
import { Badge, NativeSelect, TextField } from '../../ui';
import { RadioGroupItem } from '../../ui/RadioGroup';

export interface EngineFormValues {
  endpoint: string;
  deployment: string;
  apiKey: string;
}

interface MemoryEngineOptionProps {
  engine: MemoryEngineDescriptor;
  isActive: boolean;
  isSelected: boolean;
  /** A key is already stored for this engine (only ever true for the active one). */
  keySaved: boolean;
  /** Hosted engine while signed out: not selectable. */
  disabledReason: 'signed_out' | null;
  form: EngineFormValues;
  onFormChange: (patch: Partial<EngineFormValues>) => void;
}

/** One radio row; the configuration form opens under the selected engine. */
export default function MemoryEngineOption({
  engine,
  isActive,
  isSelected,
  keySaved,
  disabledReason,
  form,
  onFormChange,
}: MemoryEngineOptionProps) {
  const { t } = useT();
  const label = t(`memoryEngine.engine.${engine.id}.label`, engine.label);
  const description = t(`memoryEngine.engine.${engine.id}.description`, engine.description);
  const inputId = `memory-engine-${engine.id}`;
  const showKey = engine.needs_key || engine.key_optional;

  return (
    <div
      data-testid={`memory-engine-option-${engine.id}`}
      className="rounded-lg border border-line px-4 py-3">
      <div className="flex items-start gap-3">
        <RadioGroupItem
          id={inputId}
          value={engine.id}
          data-testid={`memory-engine-radio-${engine.id}`}
          disabled={disabledReason !== null}
          className="mt-0.5"
        />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <label htmlFor={inputId} className="text-sm font-medium text-content">
              {label}
            </label>
            {isActive ? <Badge variant="success">{t('memoryEngine.active')}</Badge> : null}
          </div>
          <p className="mt-0.5 text-xs leading-relaxed text-content-muted">{description}</p>
          {engine.hosted ? (
            <p className="mt-1 text-xs text-content-muted">
              {t('memoryEngine.hostedNote')}
              {disabledReason === 'signed_out' ? (
                <span className="ml-1 font-medium text-coral-600" role="note">
                  {t('memoryEngine.signInRequired')}
                </span>
              ) : null}
            </p>
          ) : null}

          {isSelected && disabledReason === null ? (
            <div className="mt-3 space-y-3" data-testid={`memory-engine-form-${engine.id}`}>
              {engine.needs_endpoint ? (
                <div>
                  <label
                    htmlFor={`${inputId}-endpoint`}
                    className="mb-1 block text-xs font-medium text-content-secondary">
                    {t('memoryEngine.endpoint')}
                  </label>
                  <TextField
                    id={`${inputId}-endpoint`}
                    mono
                    value={form.endpoint}
                    placeholder={engine.default_endpoint ?? 'https://'}
                    onChange={e => onFormChange({ endpoint: e.target.value })}
                  />
                </div>
              ) : null}
              {engine.deployments.length > 0 ? (
                <div>
                  <label
                    htmlFor={`${inputId}-deployment`}
                    className="mb-1 block text-xs font-medium text-content-secondary">
                    {t('memoryEngine.deployment')}
                  </label>
                  <NativeSelect
                    id={`${inputId}-deployment`}
                    value={form.deployment}
                    onChange={e => onFormChange({ deployment: e.target.value })}>
                    {engine.deployments.map(d => (
                      <option key={d} value={d}>
                        {t(`memoryEngine.deployment.${d}`, d)}
                      </option>
                    ))}
                  </NativeSelect>
                </div>
              ) : null}
              {showKey ? (
                <div>
                  <label
                    htmlFor={`${inputId}-key`}
                    className="mb-1 block text-xs font-medium text-content-secondary">
                    {engine.key_optional
                      ? t('memoryEngine.apiKeyOptional')
                      : t('memoryEngine.apiKey')}
                  </label>
                  <TextField
                    id={`${inputId}-key`}
                    type="password"
                    autoComplete="off"
                    value={form.apiKey}
                    placeholder={keySaved ? t('memoryEngine.keySavedPlaceholder') : ''}
                    onChange={e => onFormChange({ apiKey: e.target.value })}
                  />
                  {keySaved ? (
                    <p className="mt-1 text-xs text-content-muted">{t('memoryEngine.keySaved')}</p>
                  ) : null}
                </div>
              ) : null}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
