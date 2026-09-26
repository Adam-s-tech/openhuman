import { fireEvent, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../test/test-utils';
import AccountPanel from './AccountPanel';

const openUrlMock = vi.fn();
vi.mock('../../../utils/openUrl', () => ({
  openUrl: (...args: unknown[]) => openUrlMock(...args),
}));

const useCoreStateMock = vi.fn();
vi.mock('../../../providers/CoreStateProvider', () => ({ useCoreState: () => useCoreStateMock() }));

const useUsageStateMock = vi.fn();
vi.mock('../../../hooks/useUsageState', () => ({ useUsageState: () => useUsageStateMock() }));

vi.mock('../hooks/useSettingsNavigation', () => ({
  useSettingsNavigation: () => ({
    navigateBack: vi.fn(),
    navigateToSettings: vi.fn(),
    navigateToTeamManagement: vi.fn(),
    breadcrumbs: [],
  }),
}));

function renderPanel(currentUser: Record<string, unknown> | null) {
  useCoreStateMock.mockReturnValue({
    snapshot: { currentUser, auth: { userId: currentUser?._id ?? null } },
    clearSession: vi.fn(),
  });
  return renderWithProviders(<AccountPanel />, { preloadedState: { locale: { current: 'en' } } });
}

describe('AccountPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useUsageStateMock.mockReturnValue({ currentPlan: null });
  });

  it('renders the avatar initial and full name for a signed-in user', () => {
    renderPanel({ firstName: 'Ada', lastName: 'Lovelace', username: 'ada' });

    expect(screen.getByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('@ada')).toBeInTheDocument();
    // Avatar fallback renders the first letter of the display name.
    expect(screen.getByText('A')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open dashboard' }));
    expect(openUrlMock).toHaveBeenCalledWith('https://tinyhumans.ai/dashboard?tab=billing');
  });

  it('falls back to the username initial when no name is set', () => {
    renderPanel({ firstName: '', lastName: '', username: 'zed' });

    expect(screen.getByText('@zed')).toBeInTheDocument();
    expect(screen.getByText('Z')).toBeInTheDocument();
  });

  it('omits the identity summary block entirely when there is no user', () => {
    renderPanel(null);

    expect(screen.queryByText('@')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Open dashboard' })).not.toBeInTheDocument();
    expect(screen.queryByTestId('account-profile')).not.toBeInTheDocument();
  });

  it('shows the live plan from billing, falling back to the snapshot subscription', () => {
    useUsageStateMock.mockReturnValue({ currentPlan: { plan: 'PRO' } });
    renderPanel({ firstName: 'Ada', username: 'ada', subscription: { plan: 'FREE' } });
    expect(screen.getByTestId('account-plan')).toHaveTextContent('Pro');
  });

  it('uses the snapshot subscription plan until the live plan loads', () => {
    renderPanel({ firstName: 'Ada', username: 'ada', subscription: { plan: 'BASIC' } });
    expect(screen.getByTestId('account-plan')).toHaveTextContent('Basic');
  });

  it('omits the plan badge when no plan is known', () => {
    renderPanel({ firstName: 'Ada', username: 'ada' });
    expect(screen.queryByTestId('account-plan')).not.toBeInTheDocument();
  });
});
