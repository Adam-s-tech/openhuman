import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { MemoryHeatmap } from '../MemoryHeatmap';

function localDayAt(daysAgo: number, hour: number, minute = 0): number {
  const d = new Date(Date.now());
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() - daysAgo);
  d.setHours(hour, minute, 0, 0);
  return d.getTime();
}

function paragraphMatching(pattern: RegExp) {
  return (_content: string, node: Element | null) =>
    node?.tagName.toLowerCase() === 'p' &&
    pattern.test((node.textContent ?? '').replace(/\s+/g, ' '));
}

describe('memory overview components', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-05-16T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('renders the heatmap summary and hover tooltip', () => {
    const { container } = render(
      <MemoryHeatmap
        timestamps={[
          Math.floor(localDayAt(0, 0) / 1000),
          Math.floor(localDayAt(0, 0) / 1000),
          Math.floor(localDayAt(1, 0) / 1000),
        ]}
      />
    );

    expect(screen.getByRole('heading', { name: 'Ingestion Activity' })).toBeInTheDocument();
    expect(screen.getByText(/3 events over the last 8 months/)).toBeInTheDocument();
    expect(screen.getByText(paragraphMatching(/peak\s*:\s*2\/day/i))).toBeInTheDocument();

    const cells = container.querySelectorAll('svg rect[width="11"]');
    expect(cells.length).toBeGreaterThan(0);
    const targetCell = cells[cells.length - 1];
    expect(targetCell).toBeDefined();
    fireEvent.mouseEnter(targetCell);
    expect(screen.getByText(/events?$/)).toBeInTheDocument();
  });

  it('renders the heatmap loading skeleton', () => {
    const { container } = render(<MemoryHeatmap timestamps={[]} loading />);

    expect(screen.getByRole('heading', { name: 'Ingestion Activity' })).toBeInTheDocument();
    expect(container.querySelector('.animate-pulse')).toBeInTheDocument();
  });
});
