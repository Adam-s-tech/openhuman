import { useEffect, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

import { useT } from '../../../lib/i18n/I18nContext';
import { type AISettings, loadAISettings } from '../../../services/api/aiSettingsApi';
import CostDashboardPanel from '../../dashboard/CostDashboardPanel';
import UsageLogPanel from '../../dashboard/UsageLogPanel';
import { CenteredLoadingState, StatusLine } from '../../ui';
import SettingsTabbedPage from '../layout/SettingsTabbedPage';
import BackgroundLoopControls from './ai/BackgroundLoopControls';
import TokenUsagePanel from './TokenUsagePanel';

type TabId = 'costs' | 'log' | 'tokens' | 'background';

const TAB_HASH: Record<TabId, string> = {
  costs: '',
  log: '#log',
  tokens: '#tokens',
  background: '#background',
};

const hashToTab = (hash: string): TabId => {
  if (hash === '#background') return 'background';
  if (hash === '#tokens') return 'tokens';
  if (hash === '#log') return 'log';
  return 'costs';
};

/**
 * Connections → Usage. One page, four views as header chip tabs (the same
 * shell as Connections → LLM): the cost dashboard, the per-call usage log,
 * Tokenjuice token savings, and background loops + the credit ledger. The
 * active tab is reflected in the URL hash (`#log` / `#tokens` / `#background`)
 * so deep links and the legacy heartbeat/ledger-usage/token-usage redirects
 * land on the right view.
 */
const UsagePanel = () => {
  const { t } = useT();
  const location = useLocation();
  const navigate = useNavigate();
  // The router is the single source of truth for the active tab.
  const tab: TabId = hashToTab(location.hash);

  const selectTab = (next: TabId) => {
    navigate(`${location.pathname}${location.search}${TAB_HASH[next]}`, { replace: true });
  };

  return (
    <SettingsTabbedPage<TabId>
      title={t('settings.usage.title')}
      description={t('settings.usage.menuDesc')}
      tabsAriaLabel={t('settings.usage.title')}
      tabsTestIdPrefix="usage-tab"
      value={tab}
      onChange={selectTab}
      tabs={[
        { id: 'costs', label: t('settings.costDashboard.title') },
        { id: 'log', label: t('settings.costDashboard.usageLog') },
        { id: 'tokens', label: t('settings.tokenUsage.title') },
        { id: 'background', label: t('settings.heartbeat.title') },
      ]}>
      {tab === 'costs' && <CostDashboardPanel embedded />}
      {tab === 'log' && <UsageLogPanel />}
      {tab === 'tokens' && <TokenUsagePanel embedded />}
      {tab === 'background' && <BackgroundActivityTab />}
    </SettingsTabbedPage>
  );
};

/**
 * Background-activity tab body. Fetches the AI settings snapshot (routing map
 * + cloud providers) that BackgroundLoopControls needs — lazily, only when
 * this tab is mounted, so the default Costs tab doesn't pay for it.
 */
const BackgroundActivityTab = () => {
  const { t } = useT();
  const [snapshot, setSnapshot] = useState<AISettings | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    loadAISettings()
      .then(s => {
        if (!cancelled) setSnapshot(s);
      })
      .catch(err => {
        if (!cancelled) setLoadError(err instanceof Error ? err.message : String(err));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="space-y-4" data-testid="usage-background-tab">
      {loadError && <StatusLine saving={false} error={loadError} savingLabel="" />}
      {snapshot ? (
        <BackgroundLoopControls
          view="all"
          hideHeader
          routing={snapshot.routing}
          cloudProviders={snapshot.cloudProviders}
        />
      ) : !loadError ? (
        <CenteredLoadingState label={t('common.loading')} />
      ) : null}
    </div>
  );
};

export default UsagePanel;
