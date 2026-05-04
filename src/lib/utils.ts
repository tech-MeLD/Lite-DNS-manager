import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const providerLabels: Record<string, string> = {
  dnspod: 'DNSPod',
  cloudflare: 'Cloudflare',
  alidns: 'AliDNS',
};

export const providerColors: Record<string, string> = {
  dnspod: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  cloudflare: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  alidns: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
};

export const recordTypeColors: Record<string, string> = {
  A: 'bg-indigo-100 text-indigo-700',
  AAAA: 'bg-purple-100 text-purple-700',
  CNAME: 'bg-teal-100 text-teal-700',
  MX: 'bg-amber-100 text-amber-700',
  TXT: 'bg-slate-100 text-slate-700',
  NS: 'bg-rose-100 text-rose-700',
  SRV: 'bg-cyan-100 text-cyan-700',
  CAA: 'bg-lime-100 text-lime-700',
  SOA: 'bg-red-100 text-red-700',
  PTR: 'bg-fuchsia-100 text-fuchsia-700',
};

export function formatDate(dateStr: string | null): string {
  if (!dateStr) return '-';
  try {
    return new Date(dateStr).toLocaleString();
  } catch {
    return dateStr;
  }
}
