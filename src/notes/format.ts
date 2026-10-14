/** 时间戳 → "YYYY.MM.DD HH:mm" */
export function formatDate(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}.${p(d.getMonth() + 1)}.${p(d.getDate())} ${p(
    d.getHours(),
  )}:${p(d.getMinutes())}`;
}

/** 项目路径 → 末段文件夹名，用于在窄处（卡片、按钮）显示。 */
export function projectName(path: string): string {
  if (!path) return "";
  return path.split(/[\\/]/).pop() || path;
}
