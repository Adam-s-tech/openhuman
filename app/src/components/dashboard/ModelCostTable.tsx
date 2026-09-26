import type { ReactNode } from 'react';

import type { CostDashboardModelStats } from '../../hooks/useCostDashboard';
import { useT } from '../../lib/i18n/I18nContext';
import { Badge, DataTable, type DataTableColumn, EmptyState, Progress } from '../ui';
import { formatCurrency, formatTokens } from './formatCurrency';

interface ModelCostTableProps {
  models: CostDashboardModelStats[];
  currency: string;
  /** Card heading / hint, from the dashboard section that hosts the table. */
  title?: ReactNode;
  description?: ReactNode;
}

/** Per-model spend for the window: model, provider chip, volume, cost, share. */
const ModelCostTable = ({ models, currency, title, description }: ModelCostTableProps) => {
  const { t } = useT();

  const columns: DataTableColumn<CostDashboardModelStats>[] = [
    {
      id: 'model',
      header: t('settings.costDashboard.model'),
      // `w-full max-w-0` lets the name truncate instead of widening the table.
      className: 'w-full max-w-0',
      cell: row => {
        const modelName = row.model.includes('/')
          ? row.model.split('/').slice(1).join('/')
          : row.model;
        return (
          <div className="truncate font-medium text-content" title={row.model}>
            {modelName}
          </div>
        );
      },
    },
    {
      id: 'provider',
      header: t('settings.costDashboard.provider'),
      className: 'whitespace-nowrap',
      cell: row => <Badge>{row.provider ?? t('settings.costDashboard.unknownProvider')}</Badge>,
    },
    {
      id: 'tokens',
      header: t('settings.costDashboard.tokens'),
      align: 'right',
      className: 'whitespace-nowrap tabular-nums text-content-secondary',
      cell: row => formatTokens(row.total_tokens),
    },
    {
      id: 'requests',
      header: t('settings.costDashboard.requests'),
      align: 'right',
      className: 'whitespace-nowrap tabular-nums text-content-secondary',
      cell: row => row.request_count,
    },
    {
      id: 'cost',
      header: t('settings.costDashboard.cost'),
      align: 'right',
      className: 'whitespace-nowrap font-medium tabular-nums text-content',
      cell: row => formatCurrency(row.cost_usd, currency),
    },
    {
      id: 'share',
      header: t('settings.costDashboard.percentOfTotal'),
      align: 'right',
      className: 'whitespace-nowrap',
      cell: row => {
        const sharePct = Math.max(0, Math.min(100, row.percent_of_total));
        return (
          <div className="flex items-center justify-end gap-2">
            <Progress value={sharePct} className="h-1 w-14" />
            <span className="w-11 text-right tabular-nums text-content-secondary">
              {`${sharePct.toFixed(1)}%`}
            </span>
          </div>
        );
      },
    },
  ];

  return (
    // Sits among other dashboard cards on a scrolling page, so it does not
    // fill; past `maxHeight` its rows scroll under the pinned header.
    <DataTable<CostDashboardModelStats>
      fill={false}
      title={title}
      description={description}
      columns={columns}
      rows={models}
      rowKey={row => row.model}
      rowTestId={undefined}
      renderRow={undefined}
      ariaLabel={t('settings.costDashboard.modelBreakdown')}
      testId="model-cost-table"
      empty={
        <EmptyState
          data-testid="model-cost-table-empty"
          label={t('settings.costDashboard.noModels')}
        />
      }
      rowClassName={() => undefined}
    />
  );
};

export default ModelCostTable;
