import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Chunk, EntityRef, GraphRelation, Source } from '../../../utils/tauriCommands';
import { MemoryHeatmap } from '../MemoryHeatmap';

function localDayAt(daysAgo: number, hour: number, minute = 0): number {
  const d = new Date(Date.now());
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() - daysAgo);
  d.setHours(hour, minute, 0, 0);
  return d.getTime();
}

function makeChunk(overrides: Partial<Chunk> = {}): Chunk {
  return {
    id: 'chunk-1',
    source_kind: 'email',
    source_id: 'gmail:alice@example.com',
    owner: 'bob@example.com',
    timestamp_ms: localDayAt(0, 10),
    token_count: 100,
    lifecycle_status: 'admitted',
    content_preview: 'Memory item',
    has_embedding: true,
    tags: [],
    ...overrides,
  };
}

function makeSource(overrides: Partial<Source> = {}): Source {
  return {
    source_id: 'gmail:alice@example.com',
    display_name: 'Alice Inbox',
    source_kind: 'email',
    chunk_count: 3,
    most_recent_ms: localDayAt(0, 10),
    lifecycle_status: 'admitted',
    ...overrides,
  };
}

function makeEntity(overrides: Partial<EntityRef> = {}): EntityRef {
  return { entity_id: 'person:Alice', kind: 'person', surface: 'Alice', count: 3, ...overrides };
}

function makeRelation(overrides: Partial<GraphRelation> = {}): GraphRelation {
  return {
    namespace: 'gmail',
    subject: 'Alice',
    predicate: 'prefers',
    object: 'morning updates',
    attrs: { entity_types: { subject: 'person', object: 'preference' } },
    updatedAt: localDayAt(0, 10),
    evidenceCount: 2,
    orderIndex: null,
    documentIds: ['doc-1'],
    chunkIds: ['chunk-1'],
    ...overrides,
  };
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
