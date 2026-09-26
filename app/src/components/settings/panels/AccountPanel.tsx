import { ExternalLink } from 'lucide-react';

import { useUsageState } from '../../../hooks/useUsageState';
import { useT } from '../../../lib/i18n/I18nContext';
import { useCoreState } from '../../../providers/CoreStateProvider';
import type { PlanTier } from '../../../types/api';
import { BILLING_DASHBOARD_URL } from '../../../utils/links';
import { openUrl } from '../../../utils/openUrl';
import { AvatarFallback, AvatarRoot, Badge, Button, Card, Field } from '../../ui';
import SettingsPanel from '../layout/SettingsPanel';
import LogoutAndClearActions from '../LogoutAndClearActions';
import { PLANS } from './billingHelpers';

function planName(tier: PlanTier): string {
  return PLANS.find(plan => plan.tier === tier)?.name ?? tier;
}

/**
 * Settings → Account. One card per concern, top to bottom: who is signed in,
 * their plan and the billing dashboard, then the session and destructive
 * actions owned by {@link LogoutAndClearActions}. Sibling pages (Privacy,
 * Security, Import, …) are reached through the sidebar, not from here.
 */
const AccountPanel = () => {
  const { t } = useT();
  const { snapshot } = useCoreState();
  const user = snapshot.currentUser;
  // The usage hook fetches the live plan; the snapshot's embedded
  // subscription is the fallback until it lands (or when billing is offline).
  const { currentPlan } = useUsageState();

  const name = user ? [user.firstName, user.lastName].filter(Boolean).join(' ') || null : null;
  const username = user?.username ? `@${user.username}` : null;
  const initial = (name ?? user?.username ?? '?').slice(0, 1).toUpperCase();
  const tier: PlanTier | null = currentPlan?.plan ?? user?.subscription?.plan ?? null;

  return (
    <SettingsPanel
      testId="account-panel"
      description={t('pages.settings.accountSection.description')}>
      {user && (name || username) && (
        <Card padded data-testid="account-profile">
          <div className="flex items-center gap-4">
            <AvatarRoot className="h-12 w-12 shrink-0">
              <AvatarFallback className="bg-primary-100 text-base font-semibold text-primary-700 dark:bg-primary-500/15 dark:text-primary-300">
                {initial}
              </AvatarFallback>
            </AvatarRoot>
            <div className="min-w-0 flex-1">
              {name && <div className="truncate text-base font-semibold text-content">{name}</div>}
              {username && <div className="truncate text-sm text-content-muted">{username}</div>}
            </div>
            {tier && (
              <Badge variant={tier === 'FREE' ? 'neutral' : 'primary'} data-testid="account-plan">
                {planName(tier)}
              </Badge>
            )}
          </div>
        </Card>
      )}

      {user && (
        <Card title={t('nav.avatarMenu.billing')}>
          <Field
            label={t('settings.account.manageBilling')}
            description={t('settings.account.manageBillingDesc')}
            control={
              <Button
                variant="secondary"
                size="sm"
                trailingIcon={<ExternalLink className="h-3.5 w-3.5" aria-hidden />}
                onClick={() => void openUrl(`${BILLING_DASHBOARD_URL}?tab=billing`)}
                data-testid="account-open-billing">
                {t('settings.account.openDashboard')}
              </Button>
            }
          />
        </Card>
      )}

      <LogoutAndClearActions />
    </SettingsPanel>
  );
};

export default AccountPanel;
