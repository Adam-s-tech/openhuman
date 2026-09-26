/**
 * Sync audit history panel — shows when syncs happened, tokens consumed,
 * cost, and duration. Fetches from `openhuman.memory_sources_sync_audit_log`.
 *
 * Keeps itself current (openhuman#6257). It re-reads the history shortly after
 * any sync ends, polls while one is still running, and has a manual Refresh.
 * It used to fetch once on mount, so a run that finished while the tab was
 * open never appeared until the tab was left and re-entered.
 */
import { RefreshCw } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { useT } from '../../lib/i18n/I18nContext';
import { memorySourcesStatusList } from '../../services/memorySourcesService';
import { memorySyncAuditLog, type SyncAuditEntry } from '../../utils/tauriCommands';
import Badge from '../ui/Badge';
import Button from '../ui/Button';
import DataTable, { type DataTableColumn } from '../ui/DataTable';
import EmptyState from '../ui/EmptyState';
import { registrySyncingIds, sourceLabelsById } from './memorySourcesSyncTypes';
import { subscribeTerminalSyncEvents, useMemorySyncActivity } from './memorySyncActivityStore';

/**
 * How long after a run ends before the history is re-read. The core writes
 * the row for a run it drove before publishing the run's end, but the memory
 * driver's own periodic writer appends after its stage, so the read waits a
 * beat.
 */
export const REFETCH_AFTER_RUN_ENDS_MS = 1_000;

/** How often the history is re-read while any sync is still running. */
export const POLL_WHILE_SYNCING_MS = 10_000;

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const secs = ms / 1000;
  if (secs < 60) return `${secs.toFixed(1)}s`;
  const mins = Math.floor(secs / 60);
  const remSecs = Math.round(secs % 60);
  return `${mins}m ${remSecs}s`;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

function scopeLabel(scope: string): string {
  if (scope.startsWith('github:')) {
    return `GitHub · ${scope.slice(7)}`;
  }
  if (scope.startsWith('gmail:')) {
    return `Gmail · ${scope.slice(6).replace(/-at-/g, '@').replace(/-dot-/g, '.')}`;
  }
  if (scope.startsWith('rebuild:')) {
    return `Rebuild · ${scope.slice(8)}`;
  }
  return scope;
}

// `t` is threaded in because this is a module-level helper with no hook scope.
// The `{n}` placeholder follows the codebase's interpolation convention
// (t(...).replace('{n}', value)) — `t()` itself does not interpolate params.
export function timeAgo(iso: string, t: (key: string, fallback?: string) => string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return t('sync.timeAgo.justNow', 'just now');
  if (mins < 60) return t('sync.timeAgo.minutes', '{n}m ago').replace('{n}', String(mins));
  const hours = Math.floor(mins / 60);
  if (hours < 24) return t('sync.timeAgo.hours', '{n}h ago').replace('{n}', String(hours));
  const days = Math.floor(hours / 24);
  return t('sync.timeAgo.days', '{n}d ago').replace('{n}', String(days));
}

type SyncStatus = 'success' | 'partial' | 'failed';

const STATUS_VARIANT: Record<SyncStatus, 'success' | 'warning' | 'danger'> = {
  success: 'success',
  partial: 'warning',
  failed: 'danger',
};

const STATUS_LABEL_KEY: Record<SyncStatus, string> = {
  success: 'sync.status.success',
  partial: 'sync.status.partialShort',
  failed: 'sync.status.failed',
};

/** Date over time, with the relative age as the tooltip. */
function WhenCell({ iso, ago }: { iso: string; ago: string }) {
  const ts = Date.parse(iso);
  if (Number.isNaN(ts)) return <span title={iso}>{ago}</span>;
  const date = new Date(ts);
  return (
    <span className="flex flex-col leading-tight" title={`${date.toLocaleString()} · ${ago}`}>
      <span className="text-content">{date.toLocaleDateString()}</span>
      <span className="text-xs text-content-muted">{date.toLocaleTimeString()}</span>
    </span>
  );
}

interface SyncAuditPanelProps {
  /** Fill a height-bounded parent (only rows scroll) instead of a capped card. */
  fill?: boolean;
}

export function SyncAuditPanel({ fill = false }: SyncAuditPanelProps = {}) {
  const { t } = useT();
  const { syncingIds } = useMemorySyncActivity();
  const [entries, setEntries] = useState<SyncAuditEntry[]>([]);
  // Registry labels by source id. A row whose source has no label (removed
  // since, or a core that predates the field) keeps its scope label.
  const [labels, setLabels] = useState<Record<string, string>>({});
  // The registry's source ids. Only a run of one of them keeps the history
  // polling: the store also lights rows keyed by a document outside the
  // registry, and those never end (see `registrySyncingIds`).
  const [sourceIds, setSourceIds] = useState<ReadonlySet<string>>(() => new Set());
  const anySyncing = registrySyncingIds(syncingIds, sourceIds).length > 0;
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  // Every read — mount, a run ending, a poll, a Refresh press — is one bump of
  // this token, and the read effect below is keyed on it. A newer bump cancels
  // the read still in flight, so an older read that answers last cannot put an
  // older history back on screen.
  const [reloadToken, setReloadToken] = useState(0);
  const [query, setQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<ReadonlySet<string>>(() => new Set());

  const reload = useCallback((reason: string) => {
    console.debug('[sync-audit] reload requested reason=%s', reason);
    setReloadToken(token => token + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;
    // The history and the labels are read side by side, not together: a slow
    // status list must not hold back the rows or the Refresh button.
    void (async () => {
      console.debug('[sync-audit] load: entry token=%d', reloadToken);
      try {
        const data = await memorySyncAuditLog();
        if (cancelled) {
          console.debug('[sync-audit] load: dropped superseded token=%d', reloadToken);
          return;
        }
        setEntries(data);
        console.debug('[sync-audit] load: ok token=%d entries=%d', reloadToken, data.length);
      } catch (err) {
        console.error('[sync-audit] fetch failed', err);
      } finally {
        if (!cancelled) {
          setLoading(false);
          setRefreshing(false);
        }
      }
    })();
    // Labels and source ids only. A failed or superseded read keeps what the
    // last read that answered said, and a row with no label names itself by
    // its scope.
    void (async () => {
      try {
        const statuses = await memorySourcesStatusList();
        if (cancelled) return;
        setLabels(sourceLabelsById(statuses));
        setSourceIds(new Set(statuses.map(status => status.source_id)));
      } catch (err) {
        if (!cancelled) {
          console.warn('[sync-audit] status list failed; keeping the last labels', err);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [reloadToken]);

  // A run ending is when the history changes, whichever tab started the run.
  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const unsubscribe = subscribeTerminalSyncEvents(({ rowId, stage }) => {
      console.debug('[sync-audit] run ended rowId=%s stage=%s', rowId, stage);
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = undefined;
        reload('run-ended');
      }, REFETCH_AFTER_RUN_ENDS_MS);
    });
    return () => {
      unsubscribe();
      if (timer !== undefined) clearTimeout(timer);
    };
  }, [reload]);

  // While anything is syncing, keep reading: a run whose end the socket never
  // delivered still reaches the history on the next tick.
  useEffect(() => {
    if (!anySyncing) return undefined;
    const id = setInterval(() => {
      reload('poll');
    }, POLL_WHILE_SYNCING_MS);
    return () => clearInterval(id);
  }, [anySyncing, reload]);

  const statusOf = (e: SyncAuditEntry): SyncStatus =>
    e.success
      ? 'success'
      : (e.tree_ingest_failures ?? 0) > 0 || e.tree_error
        ? 'partial'
        : 'failed';
  const sourceName = (e: SyncAuditEntry) => labels[e.source_id] ?? scopeLabel(e.scope);

  const needle = query.trim().toLowerCase();
  const visible = entries.filter(e => {
    if (statusFilter.size > 0 && !statusFilter.has(statusOf(e))) return false;
    if (!needle) return true;
    return sourceName(e).toLowerCase().includes(needle) || e.scope.toLowerCase().includes(needle);
  });

  const totalCost = entries.reduce((s, e) => s + e.estimated_cost_usd, 0);
  const totalInput = entries.reduce((s, e) => s + e.input_tokens, 0);
  const totalOutput = entries.reduce((s, e) => s + e.output_tokens, 0);
  const summary =
    entries.length > 0 ? (
      <span className="inline-flex flex-wrap items-center gap-x-2">
        <span>
          {entries.length} {t('sync.runs', 'sync runs')}
        </span>
        <span aria-hidden>·</span>
        <span>
          {formatTokens(totalInput)} in / {formatTokens(totalOutput)} out
        </span>
        <span aria-hidden>·</span>
        <span className="font-medium text-content-secondary">
          ${totalCost.toFixed(4)} {t('sync.totalCost', 'total')}
        </span>
      </span>
    ) : undefined;

  const columns: DataTableColumn<SyncAuditEntry>[] = [
    {
      id: 'when',
      header: t('sync.when', 'When'),
      className: 'w-px whitespace-nowrap tabular-nums',
      cell: e => <WhenCell iso={e.timestamp} ago={timeAgo(e.timestamp, t)} />,
    },
    {
      id: 'source',
      header: t('sync.source', 'Source'),
      className: 'w-full max-w-0',
      cell: e => (
        <span className="block truncate" title={e.scope}>
          {sourceName(e)}
        </span>
      ),
    },
    {
      id: 'items',
      header: t('sync.items', 'Items'),
      align: 'right',
      className: 'w-px whitespace-nowrap tabular-nums',
      cell: e => e.items_fetched,
    },
    {
      id: 'tokens',
      header: t('sync.tokens', 'Tokens'),
      align: 'right',
      className: 'w-px whitespace-nowrap tabular-nums',
      cell: e => (
        <span title={`${e.input_tokens} in / ${e.output_tokens} out`}>
          {formatTokens(e.input_tokens + e.output_tokens)}
        </span>
      ),
    },
    {
      id: 'cost',
      header: t('sync.cost', 'Cost'),
      align: 'right',
      className: 'w-px whitespace-nowrap tabular-nums font-medium',
      cell: e => `$${e.estimated_cost_usd.toFixed(4)}`,
    },
    {
      id: 'duration',
      header: t('sync.duration', 'Duration'),
      align: 'right',
      className: 'w-px whitespace-nowrap tabular-nums text-content-muted',
      cell: e => formatDuration(e.duration_ms),
    },
    {
      id: 'status',
      header: t('sync.statusColumn', 'Status'),
      align: 'right',
      className: 'w-px whitespace-nowrap',
      cell: e => {
        const status = statusOf(e);
        // openhuman#5820: a fetch that committed while the memory-tree half
        // dropped items is its own "partial" verdict, not a plain failure. The
        // tooltip carries the core's error so the row explains itself.
        const title =
          status === 'success'
            ? t('sync.status.success', 'Success')
            : status === 'partial'
              ? (e.tree_error ?? t('sync.status.partial', 'Fetched, memory ingest failed'))
              : (e.error ?? t('sync.status.failed', 'Failed'));
        return (
          <Badge variant={STATUS_VARIANT[status]} title={title} data-status={status}>
            {t(STATUS_LABEL_KEY[status])}
          </Badge>
        );
      },
    },
  ];

  return (
    <DataTable<SyncAuditEntry>
      // Sits among other cards on the scrolling Sync tab: capped, scrolls inside.
      fill={fill}
      maxHeight="24rem"
      testId="sync-history-table"
      title={t('sync.auditTitle', 'Sync History')}
      description={summary}
      actions={
        <Button
          variant="secondary"
          size="sm"
          analyticsId="sync-history-refresh"
          data-testid="sync-history-refresh"
          leadingIcon={<RefreshCw className="h-3.5 w-3.5" aria-hidden />}
          disabled={refreshing}
          onClick={() => {
            setRefreshing(true);
            reload('manual');
          }}>
          {t('common.refresh', 'Refresh')}
        </Button>
      }
      search={
        entries.length > 0
          ? {
              value: query,
              onChange: setQuery,
              placeholder: t('sync.searchPlaceholder', 'Search sources…'),
              testId: 'sync-history-search',
            }
          : undefined
      }
      filters={
        entries.length > 0
          ? [
              {
                id: 'status',
                label: t('sync.statusColumn', 'Status'),
                options: (['success', 'partial', 'failed'] as const).map(value => ({
                  value,
                  label: t(STATUS_LABEL_KEY[value]),
                })),
                selected: statusFilter,
                onChange: setStatusFilter,
                testId: 'sync-history-status-filter',
              },
            ]
          : undefined
      }
      columns={columns}
      rows={visible}
      rowKey={(e, i) => `${e.timestamp}-${i}`}
      pagination={{ pageSize: 10 }}
      loading={loading}
      loadingRows={4}
      empty={
        <EmptyState
          label={
            entries.length === 0
              ? t('sync.noAuditEntries', 'No sync runs recorded yet.')
              : t('common.noResults', 'No results')
          }
        />
      }
      ariaLabel={t('sync.auditTitle', 'Sync History')}
    />
  );
}
