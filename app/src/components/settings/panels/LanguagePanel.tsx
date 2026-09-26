import { useT } from '../../../lib/i18n/I18nContext';
import LanguageSelect from '../../LanguageSelect';
import { Card, Field } from '../../ui';
import SettingsPanel from '../layout/SettingsPanel';

/**
 * Settings → Language. The display-language picker, promoted to its own page
 * from a card at the bottom of Appearance.
 */
const LanguagePanel = () => {
  const { t } = useT();

  return (
    <SettingsPanel testId="language-panel" description={t('settings.languageDesc')}>
      <Card>
        <Field
          label={t('settings.language')}
          description={t('settings.languageDesc')}
          control={<LanguageSelect ariaLabel={t('settings.language')} />}
        />
      </Card>
    </SettingsPanel>
  );
};

export default LanguagePanel;
