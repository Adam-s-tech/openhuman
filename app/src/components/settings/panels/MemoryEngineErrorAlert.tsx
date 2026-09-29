import { useNavigate } from 'react-router-dom';

import { useT } from '../../../lib/i18n/I18nContext';
import { Alert, AlertDescription, Button } from '../../ui';
import { classifyMemoryEngineError, type MemoryEngineErrorKind } from './memoryEngineUtils';

/** Friendly text for a classified engine error; generic otherwise. */
export function useMemoryEngineErrorText(): (kind: MemoryEngineErrorKind) => string {
  const { t } = useT();
  return kind =>
    kind === 'insufficient_credits'
      ? t('memoryEngine.error.insufficientCredits')
      : kind === 'session_expired'
        ? t('memoryEngine.error.sessionExpired')
        : kind === 'backend_unavailable'
          ? t('memoryEngine.error.backendUnavailable')
          : t('memoryEngine.error.generic');
}

interface MemoryEngineErrorAlertProps {
  /** The raw error (Error, string or core message). */
  error: unknown;
  /** Kind already classified by the caller; derived from `error` when omitted. */
  kind?: MemoryEngineErrorKind;
  /** Shown instead of the generic text when the error is not an engine error. */
  fallbackText?: string;
}

/**
 * The top-up / sign-in / backend-unavailable states for a memory-engine error,
 * shared by the engine panel and every memory view that surfaces core errors.
 */
export default function MemoryEngineErrorAlert({
  error,
  kind,
  fallbackText,
}: MemoryEngineErrorAlertProps) {
  const { t } = useT();
  const navigate = useNavigate();
  const errorText = useMemoryEngineErrorText();
  const resolved = kind ?? classifyMemoryEngineError(error);

  return (
    <Alert variant="destructive" data-testid={`memory-engine-error-${resolved}`}>
      <AlertDescription>
        <span>{resolved === 'other' && fallbackText ? fallbackText : errorText(resolved)}</span>
        {resolved === 'insufficient_credits' ? (
          <Button
            variant="tertiary"
            size="xs"
            className="ml-2"
            analyticsId="memory-engine-open-billing"
            data-testid="memory-engine-open-billing"
            onClick={() => navigate('/settings/account')}>
            {t('memoryEngine.error.openBilling')}
          </Button>
        ) : null}
        {resolved === 'session_expired' ? (
          <Button
            variant="tertiary"
            size="xs"
            className="ml-2"
            analyticsId="memory-engine-sign-in"
            data-testid="memory-engine-sign-in"
            onClick={() => navigate('/')}>
            {t('memoryEngine.error.signIn')}
          </Button>
        ) : null}
      </AlertDescription>
    </Alert>
  );
}
