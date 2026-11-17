export function fmtDuration(s: number | null | undefined): string {
  if (s == null || !isFinite(s)) return "—";
  const t = Math.round(s);
  const h = Math.floor(t / 3600), m = Math.floor((t % 3600) / 60), sec = t % 60;
  return [h, m, sec].map((n) => String(n).padStart(2, "0")).join(":");
}
export function fmtTimecode(s: number): string {
  const t = Math.max(0, s);
  const whole = Math.floor(t);
  const f = Math.floor((t - whole) * 100);
  return `${fmtDuration(whole)}.${String(f).padStart(2, "0")}`;
}
export function fmtBytes(n: number | null | undefined): string {
  if (n == null) return "—";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0, v = n;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(i === 0 ? 0 : 1)} ${u[i]}`;
}
export function fmtEta(s: number | null | undefined): string {
  return s == null ? "" : `${fmtDuration(s)} left`;
}
export function stem(name: string): string {
  const i = name.lastIndexOf(".");
  return i > 0 ? name.slice(0, i) : name;
}
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
