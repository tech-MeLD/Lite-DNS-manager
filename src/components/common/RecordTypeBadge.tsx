import { cn } from '../../lib/utils';
import { recordTypeColors } from '../../lib/utils';

export default function RecordTypeBadge({ type }: { type: string }) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded px-1.5 py-0.5 text-xs font-mono font-semibold',
        recordTypeColors[type] || 'bg-gray-100 text-gray-700'
      )}
    >
      {type}
    </span>
  );
}
