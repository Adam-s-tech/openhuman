/**
 * MemoryEngineErrorAlert: the shared top-up / sign-in / unavailable states that
 * the engine panel, Brain and the memory data panel reuse.
 */
import { screen } from '@testing-library/react';
import { describe, expect, test } from 'vitest';

import { renderWithProviders } from '../../../test/test-utils';
import MemoryEngineErrorAlert from './MemoryEngineErrorAlert';

describe('MemoryEngineErrorAlert', () => {
  test('a hosted 402 shows the top-up state with a billing link', () => {
    renderWithProviders(
      <MemoryEngineErrorAlert error="INSUFFICIENT_CREDITS: the memory engine is out of credits" />
    );
    expect(screen.getByTestId('memory-engine-error-insufficient_credits')).toBeInTheDocument();
    expect(screen.getByTestId('memory-engine-open-billing')).toBeInTheDocument();
  });

  test('an expired session shows the sign-in state', () => {
    renderWithProviders(
      <MemoryEngineErrorAlert error={new Error('SESSION_EXPIRED: no TinyHumans session')} />
    );
    expect(screen.getByTestId('memory-engine-error-session_expired')).toBeInTheDocument();
    expect(screen.getByTestId('memory-engine-sign-in')).toBeInTheDocument();
  });

  test('any other error shows the caller fallback text and no action', () => {
    renderWithProviders(<MemoryEngineErrorAlert error="boom" fallbackText="Graph failed" />);
    expect(screen.getByTestId('memory-engine-error-other')).toHaveTextContent('Graph failed');
    expect(screen.queryByTestId('memory-engine-open-billing')).toBeNull();
    expect(screen.queryByTestId('memory-engine-sign-in')).toBeNull();
  });
});
