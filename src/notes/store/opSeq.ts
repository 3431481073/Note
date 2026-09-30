// 操作序列（opSeq）：纯函数集合，负责摘要、转义、排序插入、事件合并与应用。
// 全部为纯函数，便于单元测试（见 test/opSeq.test.ts）。

import type { ChangedPayload, Op, PageMeta } from "../types";

/** 摘要：取前 20 个 Unicode 字符，换行替换为空格，去首尾空白。 */
export function makeSummary(content: string): string {
  const chars = Array.from(content);
  const first20 = chars.slice(0, 20).join("");
  return first20.replace(/\n/g, " ").trim();
}

/** 转义 LIKE 特殊字符，配合 SQL 端 ESCAPE '\' 使用。 */
export function escapeLike(input: string): string {
  return input
    .replace(/\\/g, "\\\\")
    .replace(/%/g, "\\%")
    .replace(/_/g, "\\_");
}

/** id 降序比较（用于 created_at / deleted_at 相同时的稳定排序）。 */
export function cmpIdDesc(a: string, b: string): number {
  if (a > b) return 1;
  if (a < b) return -1;
  return 0;
}

/** 主列表排序：created_at 降序，同刻 id 降序。cmp(a,b)>0 表示 a 应排在 b 前。 */
export function cmpActive(a: PageMeta, b: PageMeta): number {
  if (a.created_at !== b.created_at) return a.created_at - b.created_at;
  return cmpIdDesc(a.id, b.id);
}

/** 回收站排序：deleted_at 降序，同刻 id 降序。 */
export function cmpTrash(a: PageMeta, b: PageMeta): number {
  const da = a.deleted_at ?? 0;
  const db = b.deleted_at ?? 0;
  if (da !== db) return da - db;
  return cmpIdDesc(a.id, b.id);
}

/** 二分插入：list 已按 cmp 降序排序，插入 item 并返回新数组（不修改原数组）。 */
export function insertSorted<T>(
  list: readonly T[],
  item: T,
  cmp: (a: T, b: T) => number,
): T[] {
  let lo = 0;
  let hi = list.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (cmp(list[mid], item) >= 0) lo = mid + 1;
    else hi = mid;
  }
  const next = list.slice();
  next.splice(lo, 0, item);
  return next;
}

/** 根据 payload 构造一个 PageMeta（用于 create/restore 事件）。 */
function payloadToMeta(p: ChangedPayload): PageMeta {
  return {
    id: p.id,
    summary: p.summary ?? "",
    created_at: p.created_at ?? 0,
    deleted_at: p.deleted_at,
    version: p.version,
    title: p.title ?? "",
    kind: p.kind ?? "note",
    tags: p.tags ?? [],
    projects: p.projects ?? [],
  };
}

/** 就地更新列表里某条 meta 的 summary / version / deleted_at。 */
function updateInList(list: readonly PageMeta[], p: ChangedPayload): PageMeta[] {
  const idx = list.findIndex((m) => m.id === p.id);
  if (idx < 0) return list.slice();
  const next = list.slice();
  const cur = next[idx];
  next[idx] = {
    ...cur,
    summary: p.summary ?? cur.summary,
    version: p.version,
    deleted_at: p.deleted_at ?? cur.deleted_at,
  };
  return next;
}

export interface Lists {
  active: PageMeta[];
  trash: PageMeta[];
}

/**
 * 将一个后端广播事件应用到 (active, trash) 两个有序列表，返回新对象。
 * 纯函数，不修改入参。
 */
export function applyChangedPayload(lists: Lists, p: ChangedPayload): Lists {
  const { active, trash } = lists;
  switch (p.op) {
    case "create": {
      // 幂等：乐观插入后事件回声到达时不再重复插入
      if (active.some((m) => m.id === p.id)) return lists;
      return { active: insertSorted(active, payloadToMeta(p), cmpActive), trash };
    }
    case "update": {
      return { active: updateInList(active, p), trash: updateInList(trash, p) };
    }
    case "delete": {
      const meta = active.find((m) => m.id === p.id);
      if (!meta) return lists; // 已本地删除，回声忽略
      const active2 = active.filter((m) => m.id !== p.id);
      const trash2 = insertSorted(
        trash,
        { ...meta, deleted_at: p.deleted_at ?? null, version: p.version },
        cmpTrash,
      );
      return { active: active2, trash: trash2 };
    }
    case "restore": {
      const meta = trash.find((m) => m.id === p.id);
      if (!meta) return lists; // 已本地恢复，回声忽略
      const trash2 = trash.filter((m) => m.id !== p.id);
      const active2 = insertSorted(
        active,
        {
          ...meta,
          deleted_at: null,
          created_at: p.created_at ?? meta.created_at,
          version: p.version,
        },
        cmpActive,
      );
      return { active: active2, trash: trash2 };
    }
    case "purge": {
      if (!trash.some((m) => m.id === p.id)) return lists;
      return { active, trash: trash.filter((m) => m.id !== p.id) };
    }
  }
}

export interface OpEvent {
  op: Op;
  id: string;
  version: number;
  summary?: string | null;
  created_at?: number | null;
  deleted_at?: number | null;
}

/**
 * 合并同一 id 的一串操作（实现 §8.4 合并规则表）。
 * 返回合并后的单条操作，或 null 表示净效果为零、无需广播。
 */
export function mergeOps(ops: readonly OpEvent[]): OpEvent | null {
  let acc: OpEvent | null = null;
  for (const op of ops) {
    if (acc === null) {
      acc = { ...op };
      continue;
    }
    // 相邻抵消
    if (acc.op === "create" && op.op === "delete") {
      acc = null;
      continue;
    }
    if (acc.op === "delete" && op.op === "restore") {
      acc = null;
      continue;
    }
    // create/restore 后被 update，保留"创建/恢复"语义
    if (acc.op === "create" && op.op === "update") {
      acc = { ...op, op: "create" };
      continue;
    }
    if (acc.op === "restore" && op.op === "update") {
      acc = { ...op, op: "restore" };
      continue;
    }
    // 其余取最后一次操作
    acc = { ...op };
  }
  return acc;
}
