import { useT } from '../../../../lib/i18n/I18nContext';
import { Button, Label, TextArea } from '../../../ui';
import { useSettingsNavigation } from '../../hooks/useSettingsNavigation';
import { applyPersonaField, parsePersonaFields, type PersonaFieldKey } from './personaSections';

interface PersonaGuidedFieldsProps {
  /** The raw SOUL.md text — the single source of truth this view edits. */
  value: string;
  /** Emit the updated SOUL.md text after a managed section is spliced. */
  onChange: (nextSoul: string) => void;
  disabled?: boolean;
}

interface FieldDef {
  key: PersonaFieldKey;
  labelKey: string;
  placeholderKey: string;
  testId: string;
}

const FIELDS: readonly FieldDef[] = [
  {
    key: 'personality',
    labelKey: 'settings.persona.builder.personalityLabel',
    placeholderKey: 'settings.persona.builder.personalityPlaceholder',
    testId: 'persona-guided-personality',
  },
  {
    key: 'voice',
    labelKey: 'settings.persona.builder.voiceLabel',
    placeholderKey: 'settings.persona.builder.voicePlaceholder',
    testId: 'persona-guided-voice',
  },
  {
    key: 'about',
    labelKey: 'settings.persona.builder.aboutLabel',
    placeholderKey: 'settings.persona.builder.aboutPlaceholder',
    testId: 'persona-guided-about',
  },
] as const;

/**
 * Structured persona editor (issue #4253, PR1). The template picker is a
 * separate card on the Personality page. Presents a few friendly fields
 * that map to named `SOUL.md` sections so non-technical users never touch raw
 * markdown. The raw text stays the source of truth: each edit is spliced back
 * into `value` via {@link applyPersonaField} and emitted through `onChange`.
 */
const PersonaGuidedFields = ({ value, onChange, disabled = false }: PersonaGuidedFieldsProps) => {
  const { t } = useT();
  const { navigateToSettings } = useSettingsNavigation();
  const fields = parsePersonaFields(value);

  return (
    <div className="space-y-4">
      <p className="text-xs text-content-muted leading-relaxed">
        {t('settings.persona.builder.intro')}
      </p>

      {FIELDS.map(field => (
        <div key={field.key}>
          <Label htmlFor={field.testId}>{t(field.labelKey)}</Label>
          <TextArea
            id={field.testId}
            data-testid={field.testId}
            aria-label={t(field.labelKey)}
            value={fields[field.key]}
            rows={3}
            disabled={disabled}
            placeholder={t(field.placeholderKey)}
            onChange={e => onChange(applyPersonaField(value, field.key, e.target.value))}
            className="mt-1.5"
          />
        </div>
      ))}

      <p className="text-xs text-content-muted leading-relaxed">
        {t('settings.persona.builder.preservedNote')} {t('settings.persona.builder.securityNote')}{' '}
        <Button
          variant="tertiary"
          size="xs"
          data-testid="persona-guided-agent-access"
          className="h-auto w-auto p-0 text-primary-700 hover:bg-transparent hover:underline dark:text-primary-300"
          onClick={() => navigateToSettings('agent-access')}>
          {t('settings.persona.builder.securityLink')}
        </Button>
      </p>
    </div>
  );
};

export default PersonaGuidedFields;
