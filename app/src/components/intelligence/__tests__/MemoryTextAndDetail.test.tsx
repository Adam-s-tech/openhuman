import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { MemoryRetrievalEntity } from '../../../utils/tauriCommands';
import { MemoryTextWithEntities } from '../MemoryTextWithEntities';

const rpcMocks = vi.hoisted(() => ({
  memoryTreeEntityIndexFor: vi.fn(),
  memoryTreeChunkScore: vi.fn(),
}));

vi.mock('../../../utils/tauriCommands', async () => {
  const actual = await vi.importActual<typeof import('../../../utils/tauriCommands')>(
    '../../../utils/tauriCommands'
  );
  return {
    ...actual,
    memoryTreeEntityIndexFor: rpcMocks.memoryTreeEntityIndexFor,
    memoryTreeChunkScore: rpcMocks.memoryTreeChunkScore,
  };
});

describe('MemoryTextWithEntities', () => {
  it('renders nothing when there is no text or entity chip data', () => {
    const { container } = render(<MemoryTextWithEntities text="" />);

    expect(container).toBeEmptyDOMElement();
  });

  it('renders structured entity chips with type titles', () => {
    const entities: MemoryRetrievalEntity[] = [
      { id: 'e1', name: 'Alice', entity_type: 'PERSON' },
      { id: 'e2', name: 'Atlas', entity_type: 'PROJECT' },
    ];

    render(<MemoryTextWithEntities text="Relevant context" entities={entities} />);

    expect(screen.getByTitle('Alice (PERSON)')).toBeInTheDocument();
    expect(screen.getByTitle('Atlas (PROJECT)')).toBeInTheDocument();
    expect(screen.getByText('Relevant context')).toBeInTheDocument();
  });

  it('turns inline entity annotations into accessible badges', () => {
    render(<MemoryTextWithEntities text="Alice (PERSON) owns Atlas (PROJECT)." />);

    expect(screen.getByTitle('Entity type: PERSON')).toHaveTextContent('PERSON');
    expect(screen.getByTitle('Entity type: PROJECT')).toHaveTextContent('PROJECT');
    expect(screen.getByText(/Alice/)).toBeInTheDocument();
    expect(screen.getByText(/owns Atlas/)).toBeInTheDocument();
  });
});
