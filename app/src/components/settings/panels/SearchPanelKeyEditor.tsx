import { useId } from 'react';

import Button from '../../ui/Button';
import Input from '../../ui/Input';

export interface KeyEditorProps {
  label: string;
  placeholder: string;
  show: boolean;
  onToggleShow: () => void;
  value: string;
  onChange: (v: string) => void;
  onSave: () => void;
  onClear: () => void;
  configured: boolean;
  /** Where to get a key; the link is hidden when the provider has none. */
  docUrl?: string | null;
  disabled?: boolean;
  testId?: string;
  t: (key: string) => string;
}

/** One API-key row: label + doc link, a maskable input, and save/clear actions. */
const KeyEditor = ({
  label,
  placeholder,
  show,
  onToggleShow,
  value,
  onChange,
  onSave,
  onClear,
  configured,
  docUrl,
  disabled = false,
  testId,
  t,
}: KeyEditorProps) => {
  const inputId = useId();

  return (
    <div
      role="group"
      aria-labelledby={inputId}
      data-testid={testId}
      className="rounded-lg border border-line-subtle bg-surface-subtle p-3">
      <div className="flex items-center justify-between mb-2">
        <label
          id={inputId}
          htmlFor={`${inputId}-input`}
          className="text-xs font-semibold text-content">
          {label}
        </label>
        {docUrl && (
          <a
            href={docUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-[10px] text-primary-500 hover:underline">
            {t('settings.search.getApiKey')} ↗
          </a>
        )}
      </div>
      <div className="flex items-center gap-2">
        <Input
          id={`${inputId}-input`}
          type={show ? 'text' : 'password'}
          inputSize="sm"
          value={value}
          onChange={e => onChange(e.target.value)}
          placeholder={placeholder}
          className="flex-1 min-w-0 font-mono"
        />
        <Button type="button" variant="secondary" size="xs" onClick={onToggleShow}>
          {show ? t('settings.search.hide') : t('settings.search.show')}
        </Button>
        <Button
          type="button"
          variant="primary"
          size="xs"
          onClick={onSave}
          disabled={disabled || value.trim().length === 0}>
          {t('settings.search.save')}
        </Button>
        {configured && (
          <Button
            type="button"
            variant="secondary"
            tone="danger"
            size="xs"
            onClick={onClear}
            disabled={disabled}>
            {t('settings.search.clear')}
          </Button>
        )}
      </div>
    </div>
  );
};

export default KeyEditor;
