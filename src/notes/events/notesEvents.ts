// 事件监听 + 季节主题/水印加载。
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { notesApi } from "../store/notesApi";
import { notesStore } from "../store/notesStore";
import type { ChangedPayload, Stats } from "../types";

interface MonthConfig {
  src: string;
  ink: string;
  opacity?: number;
  fit?: string;
  align?: string;
}

interface SeasonsConfig {
  version: number;
  defaultOpacity: number;
  defaultFit: string;
  defaultAlign: string;
  months: Record<string, MonthConfig>;
}

export async function loadSeasonsConfig(): Promise<SeasonsConfig> {
  return invoke<SeasonsConfig>("read_seasons_config");
}

/** 应用指定月份（1–12）的主题色与水印。 */
export async function applySeason(month: number) {
  let cfg: SeasonsConfig;
  try {
    cfg = await loadSeasonsConfig();
  } catch (e) {
    console.warn("load seasons config failed", e);
    return;
  }
  const m = cfg.months[String(month)];
  if (!m) return;

  document.documentElement.style.setProperty("--ink-current", m.ink);

  const boxes = document.querySelectorAll(".paper-watermark");
  if (boxes.length === 0) return;

  try {
    const configDir = await notesApi.getConfigDir();
    const absPath = `${configDir}/${m.src}`;
    const url = `${convertFileSrc(absPath)}?t=${Date.now()}`;
    for (const box of boxes) {
      const el = box as HTMLElement;
      el.style.opacity = String(m.opacity ?? cfg.defaultOpacity);
      el.innerHTML = "";
      const img = document.createElement("img");
      img.alt = "";
      img.style.objectFit = m.fit ?? cfg.defaultFit;
      img.style.objectPosition = m.align ?? cfg.defaultAlign;
      img.src = url;
      img.onerror = () => console.warn("watermark load failed:", m.src);
      el.appendChild(img);
    }
  } catch (e) {
    console.warn("watermark resolve failed", e);
  }
}

let unlisteners: (() => void)[] | null = null;

/** 挂载后端广播事件监听。 */
export async function setupNotesEvents() {
  if (unlisteners) return;
  const u1 = await listen<ChangedPayload>("notes://changed", (e) =>
    notesStore.handleChanged(e.payload),
  );
  const u2 = await listen<Stats>("notes://stats", (e) =>
    notesStore.setStats(e.payload),
  );
  const u3 = await listen("notes://config-changed", () => {
    applySeason(new Date().getMonth() + 1);
  });
  unlisteners = [u1, u2, u3];
}
