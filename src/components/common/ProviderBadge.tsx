import { cn } from '../../lib/utils';
import { providerLabels, providerColors } from '../../lib/utils';
import type { ProviderType } from '../../types';

export default function ProviderBadge({ provider }: { provider: ProviderType }) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium',
        providerColors[provider] || 'bg-gray-100 text-gray-700'
      )}
    >
      {providerLabels[provider] || provider}
    </span>
  );
}
