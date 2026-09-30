// 视图结构树（outline）的纯函数操作：不可变更新 + 查找。
import type { OutlineNode } from "../types";

let seq = 0;

export function newId(): string {
  seq += 1;
  return `n_${Date.now().toString(36)}_${seq}_${Math.random().toString(36).slice(2, 6)}`;
}

export function newNode(title = "", x?: number, y?: number): OutlineNode {
  return {
    id: newId(),
    title,
    text: "",
    tags: [],
    projects: [],
    addedToNotes: false,
    noteId: null,
    collapsed: false,
    children: [],
    x,
    y,
  };
}

export function findNode(nodes: OutlineNode[], id: string): OutlineNode | null {
  for (const n of nodes) {
    if (n.id === id) return n;
    const found = findNode(n.children, id);
    if (found) return found;
  }
  return null;
}

/** 对整棵树做不可变映射（浅拷贝路径上的每个节点）。 */
export function mapTree(nodes: OutlineNode[], fn: (n: OutlineNode) => OutlineNode): OutlineNode[] {
  return nodes.map((n) => fn({ ...n, children: mapTree(n.children, fn) }));
}

export function addRoot(nodes: OutlineNode[], child: OutlineNode): OutlineNode[] {
  return [...nodes, child];
}

export function addChild(nodes: OutlineNode[], parentId: string, child: OutlineNode): OutlineNode[] {
  return mapTree(nodes, (n) => (n.id === parentId ? { ...n, children: [...n.children, child] } : n));
}

export function removeNode(nodes: OutlineNode[], id: string): OutlineNode[] {
  return nodes
    .filter((n) => n.id !== id)
    .map((n) => ({ ...n, children: removeNode(n.children, id) }));
}

export function updateNode(
  nodes: OutlineNode[],
  id: string,
  patch: Partial<OutlineNode>,
): OutlineNode[] {
  return mapTree(nodes, (n) => (n.id === id ? { ...n, ...patch } : n));
}

/** 把整棵结构树导出为纯文本：节点名 + 节点内容，缩进表示层级，标注清楚。 */
export function outlineToText(nodes: OutlineNode[]): string {
  const lines: string[] = [];
  const walk = (n: OutlineNode, depth: number) => {
    const indent = "  ".repeat(depth);
    lines.push(`${indent}■ 节点：${n.title || "（未命名）"}`);
    if (n.text) {
      const textLines = n.text.split("\n");
      lines.push(`${indent}  节点内容：${textLines[0]}`);
      for (const line of textLines.slice(1)) {
        lines.push(`${indent}    ${line}`);
      }
    }
    n.children.forEach((c) => walk(c, depth + 1));
  };
  nodes.forEach((n) => walk(n, 0));
  return lines.join("\n");
}

/** 平铺所有节点（DFS，父在子前）。 */
export function flattenNodes(nodes: OutlineNode[]): OutlineNode[] {
  const out: OutlineNode[] = [];
  const walk = (n: OutlineNode) => {
    out.push(n);
    n.children.forEach(walk);
  };
  nodes.forEach(walk);
  return out;
}

// 树布局常量（画布内像素）。坐标是「节点中心」。
const NODE_H = 36;
const LEVEL_H = 88; // 上下层间距
const HORIZ = 150; // 每个叶子占的水平宽度

/**
 * 计算上→下树布局：父节点在上，子节点在下、水平铺开。
 * 每个叶子占一个「叶槽」下标，内部节点取子节点叶槽的中心。
 * 已有 x/y 的节点（用户拖过）保留，没有的自动排。
 */
export function layoutTree(nodes: OutlineNode[]): { id: string; x: number; y: number }[] {
  const out: { id: string; x: number; y: number }[] = [];
  // 1) 叶子槽下标（内部节点 = 首尾子节点叶槽中心）。
  let leaf = 0;
  const center = new Map<string, number>();
  const assign = (n: OutlineNode) => {
    if (n.children.length === 0) {
      center.set(n.id, leaf);
      leaf++;
    } else {
      n.children.forEach(assign);
      const cs = n.children.map((c) => center.get(c.id)!);
      center.set(n.id, (cs[0] + cs[cs.length - 1]) / 2);
    }
  };
  nodes.forEach(assign);
  // 2) 深度。
  const depth = new Map<string, number>();
  const setDepth = (n: OutlineNode, d: number) => {
    depth.set(n.id, d);
    n.children.forEach((c) => setDepth(c, d + 1));
  };
  nodes.forEach((n) => setDepth(n, 0));
  // 3) 中心坐标。
  for (const n of flattenNodes(nodes)) {
    const c = center.get(n.id) ?? 0;
    const d = depth.get(n.id) ?? 0;
    if (n.x == null || n.y == null) {
      out.push({ id: n.id, x: 40 + c * HORIZ, y: 40 + d * LEVEL_H });
    } else {
      out.push({ id: n.id, x: n.x, y: n.y });
    }
  }
  return out;
}
