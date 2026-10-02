import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { describe, expect, it } from 'vitest';

import BrainRedirect from './BrainRedirect';

function Where() {
  const { pathname, search, hash } = useLocation();
  return <div data-testid="where">{`${pathname}${search}${hash}`}</div>;
}

function renderAt(entry: string) {
  render(
    <MemoryRouter initialEntries={[entry]}>
      <Routes>
        <Route path="/brain" element={<BrainRedirect />} />
        <Route path="/connections" element={<Where />} />
      </Routes>
    </MemoryRouter>
  );
  return screen.getByTestId('where').textContent;
}

describe('BrainRedirect', () => {
  it('lands bare /brain on the Brain tab of Connections', () => {
    expect(renderAt('/brain')).toBe('/connections?tab=brain');
  });

  it('moves the old ?tab= sub-tab to ?brain=', () => {
    expect(renderAt('/brain?tab=sources')).toBe('/connections?tab=brain&brain=sources');
  });

  it('keeps other params and the hash', () => {
    expect(renderAt('/brain?tab=sync&view=history#x')).toBe(
      '/connections?tab=brain&brain=sync&view=history#x'
    );
  });
});
