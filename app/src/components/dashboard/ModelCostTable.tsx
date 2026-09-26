import type { CostDashboardModelStats } from '../../hooks/useCostDashboard';
import { useT } from '../../lib/i18n/I18nContext';
import { Badge, EmptyState, Progress } from '../ui';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/Table';
import { formatCurrency, formatTokens } from './formatCurrency';

interface ModelCostTableProps {
  models: CostDashboardModelStats[];
  currency: string;
}

/** Per-model spend for the window: model, provider chip, volume, cost, share. */
const ModelCostTable = ({ models, currency }: ModelCostTableProps) => {
  const { t } = useT();
  if (models.length === 0) {
    return (
      <EmptyState
        data-testid="model-cost-table-empty"
        label={t('settings.costDashboard.noModels')}
      />
    );
  }

  return (
    <div data-testid="model-cost-table">
      <Table>
        <TableHeader>
          <TableRow className="hover:bg-transparent">
            <TableHead className="w-full">{t('settings.costDashboard.model')}</TableHead>
            <TableHead>{t('settings.costDashboard.provider')}</TableHead>
            <TableHead className="text-right">{t('settings.costDashboard.tokens')}</TableHead>
            <TableHead className="text-right">{t('settings.costDashboard.requests')}</TableHead>
            <TableHead className="text-right">{t('settings.costDashboard.cost')}</TableHead>
            <TableHead className="text-right">
              {t('settings.costDashboard.percentOfTotal')}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {models.map(row => {
            const modelName = row.model.includes('/')
              ? row.model.split('/').slice(1).join('/')
              : row.model;
            const sharePct = Math.max(0, Math.min(100, row.percent_of_total));
            return (
              <TableRow key={row.model} data-testid={`model-row-${row.model}`}>
                {/* `w-full max-w-0` lets the name truncate instead of widening the table. */}
                <TableCell className="w-full max-w-0">
                  <div className="truncate font-medium text-content" title={row.model}>
                    {modelName}
                  </div>
                </TableCell>
                <TableCell className="whitespace-nowrap">
                  <Badge>{row.provider ?? t('settings.costDashboard.unknownProvider')}</Badge>
                </TableCell>
                <TableCell className="whitespace-nowrap text-right tabular-nums text-content-secondary">
                  {formatTokens(row.total_tokens)}
                </TableCell>
                <TableCell className="whitespace-nowrap text-right tabular-nums text-content-secondary">
                  {row.request_count}
                </TableCell>
                <TableCell className="whitespace-nowrap text-right font-medium tabular-nums text-content">
                  {formatCurrency(row.cost_usd, currency)}
                </TableCell>
                <TableCell className="whitespace-nowrap">
                  <div className="flex items-center justify-end gap-2">
                    <Progress value={sharePct} className="h-1 w-14" />
                    <span className="w-11 text-right tabular-nums text-content-secondary">
                      {`${sharePct.toFixed(1)}%`}
                    </span>
                  </div>
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </div>
  );
};

export default ModelCostTable;
