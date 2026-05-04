import { Loader2 } from 'lucide-react';
import { cn } from '../../lib/utils';

export default function LoadingSpinner({
  size = 'default',
  className,
}: {
  size?: 'sm' | 'default' | 'lg';
  className?: string;
}) {
  const sizeClass = {
    sm: 'h-4 w-4',
    default: 'h-6 w-6',
    lg: 'h-10 w-10',
  }[size];

  return (
    <div className={cn('flex items-center justify-center p-8', className)}>
      <Loader2 className={cn('animate-spin text-muted-foreground', sizeClass)} />
    </div>
  );
}
