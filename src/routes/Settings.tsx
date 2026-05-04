import { useTheme } from '../context/ThemeContext';
import { Moon, Sun, Monitor } from 'lucide-react';

export default function Settings() {
  const { theme, setTheme } = useTheme();

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Settings</h1>
        <p className="text-sm text-muted-foreground mt-1">
          Application preferences
        </p>
      </div>

      <div className="rounded-lg border border-border">
        <div className="border-b border-border px-6 py-4">
          <h2 className="text-base font-semibold text-foreground">Appearance</h2>
          <p className="text-sm text-muted-foreground mt-0.5">
            Choose your preferred theme
          </p>
        </div>
        <div className="p-6">
          <div className="flex gap-3">
            <ThemeButton
              current={theme}
              value="light"
              icon={<Sun className="h-5 w-5" />}
              label="Light"
              onClick={() => setTheme('light')}
            />
            <ThemeButton
              current={theme}
              value="dark"
              icon={<Moon className="h-5 w-5" />}
              label="Dark"
              onClick={() => setTheme('dark')}
            />
            <ThemeButton
              current={theme}
              value="system"
              icon={<Monitor className="h-5 w-5" />}
              label="System"
              onClick={() => setTheme('system')}
            />
          </div>
        </div>
      </div>

      <div className="rounded-lg border border-border">
        <div className="border-b border-border px-6 py-4">
          <h2 className="text-base font-semibold text-foreground">About</h2>
        </div>
        <div className="p-6">
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Application</span>
              <span className="text-foreground font-medium">DNS Manager</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Version</span>
              <span className="text-foreground font-medium">0.1.0</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Platform</span>
              <span className="text-foreground font-medium">Windows</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Providers Supported</span>
              <span className="text-foreground font-medium">DNSPod, Cloudflare, AliDNS</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ThemeButton({
  current,
  value,
  icon,
  label,
  onClick,
}: {
  current: string;
  value: string;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  const isActive = current === value;
  return (
    <button
      onClick={onClick}
      className={`flex flex-col items-center gap-2 rounded-lg border-2 p-4 min-w-[100px] transition-all ${
        isActive
          ? 'border-primary bg-primary/5 text-primary'
          : 'border-border text-muted-foreground hover:bg-accent'
      }`}
    >
      {icon}
      <span className="text-sm font-medium">{label}</span>
    </button>
  );
}
