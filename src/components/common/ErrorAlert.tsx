import { AlertTriangle, RefreshCw } from 'lucide-react';

interface ErrorAlertProps {
  message: string;
  onRetry?: () => void;
  onDismiss?: () => void;
}

export default function ErrorAlert({ message, onRetry, onDismiss }: ErrorAlertProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 rounded-lg border border-destructive/50 bg-destructive/5 p-8 text-center">
      <AlertTriangle className="h-8 w-8 text-destructive" />
      <p className="text-sm text-destructive">{message}</p>
      <div className="flex gap-2">
        {onRetry && (
          <button
            onClick={onRetry}
            className="inline-flex items-center gap-1 rounded-md bg-destructive/10 px-3 py-1.5 text-sm font-medium text-destructive hover:bg-destructive/20 transition-colors"
          >
            <RefreshCw className="h-3 w-3" />
            Retry
          </button>
        )}
        {onDismiss && (
          <button
            onClick={onDismiss}
            className="rounded-md border border-destructive/30 px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent transition-colors"
          >
            Dismiss
          </button>
        )}
      </div>
    </div>
  );
}
