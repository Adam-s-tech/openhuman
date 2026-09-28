import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../test/test-utils';
import ComputerPanel, { ComputerModelsSection } from './ComputerPanel';

const mocks = vi.hoisted(() => ({ getConfig: vi.fn(), update: vi.fn() }));
vi.mock('../../../lib/i18n/I18nContext', () => ({ useT: () => ({ t: (key: string) => key }) }));
vi.mock('../../../utils/tauriCommands/config', () => ({
  openhumanGetConfig: mocks.getConfig,
  openhumanUpdateComputerSettings: mocks.update,
}));
vi.mock('../../desktop/DesktopConnectionPage', () => ({
  default: ({ embedded }: { embedded?: boolean }) => (
    <div data-testid="computer-desktop" data-embedded={String(embedded)} />
  ),
}));
vi.mock('./BrowserConnectionsPanel', () => ({
  default: ({ embedded }: { embedded?: boolean }) => (
    <div data-testid="computer-browser" data-embedded={String(embedded)} />
  ),
}));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.getConfig.mockResolvedValue({
    result: { config: { computer: { decision_model: 'jev', sage_fast: false } } },
  });
  mocks.update.mockResolvedValue({ result: { config: {} } });
});

describe('ComputerPanel', () => {
  it('hosts desktop and browser as embedded sub-tabs', () => {
    const onSectionChange = vi.fn();
    renderWithProviders(<ComputerPanel onSectionChange={onSectionChange} />);
    expect(screen.getByTestId('computer-desktop')).toHaveAttribute('data-embedded', 'true');
    fireEvent.click(screen.getByTestId('computer-tab-browser'));
    expect(screen.getByTestId('computer-browser')).toHaveAttribute('data-embedded', 'true');
    expect(onSectionChange).toHaveBeenCalledWith('browser');
  });

  it('follows a controlled section', () => {
    renderWithProviders(<ComputerPanel section="models" />);
    expect(screen.getByTestId('computer-models')).toBeInTheDocument();
  });
});

describe('ComputerModelsSection', () => {
  it('saves the decision model, sage fast mode and rescue settings', async () => {
    renderWithProviders(<ComputerModelsSection />);
    await waitFor(() => expect(mocks.getConfig).toHaveBeenCalled());
    fireEvent.change(screen.getByLabelText('computer.models.decisionModel'), {
      target: { value: 'sage' },
    });
    fireEvent.click(screen.getByLabelText('computer.models.sageFast'));
    fireEvent.change(screen.getByLabelText('computer.models.rescueModel'), {
      target: { value: 'openai/gpt-6-luna' },
    });
    fireEvent.change(screen.getByLabelText('computer.models.maxRescues'), {
      target: { value: '2' },
    });
    fireEvent.click(screen.getByText('common.save'));
    await waitFor(() =>
      expect(mocks.update).toHaveBeenCalledWith({
        decision_model: 'sage',
        sage_fast: true,
        planner_model: '',
        rescue_model: 'openai/gpt-6-luna',
        max_rescues: 2,
      })
    );
    expect(await screen.findByText('computer.models.saved')).toBeInTheDocument();
  });

  it('refuses rescues outside 0 to 5 without saving', async () => {
    renderWithProviders(<ComputerModelsSection />);
    await waitFor(() => expect(mocks.getConfig).toHaveBeenCalled());
    fireEvent.change(screen.getByLabelText('computer.models.maxRescues'), {
      target: { value: '9' },
    });
    fireEvent.click(screen.getByText('common.save'));
    expect(await screen.findByText('computer.models.rescuesBounds')).toBeInTheDocument();
    expect(mocks.update).not.toHaveBeenCalled();
  });

  it('shows a load error', async () => {
    mocks.getConfig.mockRejectedValueOnce(new Error('core offline'));
    renderWithProviders(<ComputerModelsSection />);
    expect(await screen.findByText('core offline')).toBeInTheDocument();
  });
});
