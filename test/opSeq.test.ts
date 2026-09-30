import { describe, expect, it } from "vitest";
import {
  applyChangedPayload,
  cmpActive,
  cmpTrash,
  escapeLike,
  insertSorted,
  makeSummary,
  mergeOps,
  type OpEvent,
} from "../src/notes/store/opSeq";
import type { ChangedPayload, PageMeta } from "../src/notes/types";

function meta(id: string, created_at: number, deleted_at: number | null = null): PageMeta {
  return { id, summary: id, created_at, deleted_at, version: 1, title: "", kind: "note", tags: [], projects: [] };
}

describe("makeSummary", () => {
  it("取前 20 字符并替换换行", () => {
    expect(makeSummary("a".repeat(100)).length).toBe(20);
    expect(makeSummary("ab\ncd")).toBe("ab cd");
    expect(makeSummary("  hi  ")).toBe("hi");
  });
});

describe("escapeLike", () => {
  it("转义 \\ % _", () => {
    expect(escapeLike("a\\b%c_d")).toBe("a\\\\b\\%c\\_d");
  });
});

describe("insertSorted / cmp", () => {
  it("created_at 降序插入", () => {
    let list: PageMeta[] = [meta("a", 30), meta("b", 10)];
    list = insertSorted(list, meta("c", 20), cmpActive);
    expect(list.map((m) => m.id)).toEqual(["a", "c", "b"]);
  });
  it("deleted_at 降序插入", () => {
    let list: PageMeta[] = [meta("a", 0, 30), meta("b", 0, 10)];
    list = insertSorted(list, meta("c", 0, 20), cmpTrash);
    expect(list.map((m) => m.id)).toEqual(["a", "c", "b"]);
  });
});

describe("mergeOps（§8.4 合并规则表）", () => {
  const op = (o: OpEvent["op"], version: number, summary = "s"): OpEvent => ({
    op: o,
    id: "x",
    version,
    summary,
  });

  it("create + update → create", () => {
    const r = mergeOps([op("create", 1), op("update", 2)]);
    expect(r?.op).toBe("create");
    expect(r?.version).toBe(2);
  });
  it("create + delete → 净零", () => {
    expect(mergeOps([op("create", 1), op("delete", 2)])).toBeNull();
  });
  it("update + update → update", () => {
    expect(mergeOps([op("update", 2), op("update", 3)])?.op).toBe("update");
  });
  it("update + delete → delete", () => {
    expect(mergeOps([op("update", 2), op("delete", 3)])?.op).toBe("delete");
  });
  it("delete + restore → 净零", () => {
    expect(mergeOps([op("delete", 2), op("restore", 3)])).toBeNull();
  });
  it("delete + purge → purge", () => {
    expect(mergeOps([op("delete", 2), op("purge", 3)])?.op).toBe("purge");
  });
  it("restore + update → restore", () => {
    expect(mergeOps([op("restore", 2), op("update", 3)])?.op).toBe("restore");
  });
  it("create + delete + restore → restore", () => {
    const r = mergeOps([op("create", 1), op("delete", 2), op("restore", 3)]);
    expect(r?.op).toBe("restore");
    expect(r?.version).toBe(3);
  });
});

describe("applyChangedPayload（幂等）", () => {
  const payload = (p: Partial<ChangedPayload>): ChangedPayload => ({
    op: "create",
    id: "x",
    version: 1,
    view: "active",
    created_at: 100,
    deleted_at: null,
    summary: "hello",
    ts: 0,
    ...p,
  });

  it("create 插入 active", () => {
    const r = applyChangedPayload({ active: [], trash: [] }, payload({}));
    expect(r.active.map((m) => m.id)).toEqual(["x"]);
  });
  it("create 幂等（重复事件不重复插入）", () => {
    const once = applyChangedPayload({ active: [], trash: [] }, payload({}));
    const twice = applyChangedPayload(once, payload({}));
    expect(twice.active.length).toBe(1);
  });
  it("delete 从 active 移到 trash", () => {
    const init = applyChangedPayload({ active: [], trash: [] }, payload({}));
    const r = applyChangedPayload(init, {
      op: "delete",
      id: "x",
      version: 2,
      view: "trash",
      created_at: 100,
      deleted_at: 200,
      summary: null,
      ts: 0,
    });
    expect(r.active.length).toBe(0);
    expect(r.trash.map((m) => m.id)).toEqual(["x"]);
    expect(r.trash[0].deleted_at).toBe(200);
  });
  it("restore 从 trash 移回 active", () => {
    const afterDelete = {
      active: [] as PageMeta[],
      trash: [meta("x", 100, 200)],
    };
    const r = applyChangedPayload(afterDelete, {
      op: "restore",
      id: "x",
      version: 3,
      view: "active",
      created_at: 100,
      deleted_at: null,
      summary: "hello",
      ts: 0,
    });
    expect(r.trash.length).toBe(0);
    expect(r.active.map((m) => m.id)).toEqual(["x"]);
    expect(r.active[0].deleted_at).toBeNull();
  });
  it("purge 从 trash 移除", () => {
    const r = applyChangedPayload(
      { active: [], trash: [meta("x", 100, 200)] },
      payload({ op: "purge", id: "x", view: "trash" }),
    );
    expect(r.trash.length).toBe(0);
  });
});
