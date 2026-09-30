import { useEffect, useMemo, useRef, useState } from "react";
import type { OutlineNode } from "../types";
import { flattenNodes, layoutTree } from "../store/tree";

const NODE_H = 36;

interface OutlineTreeProps {
  nodes: OutlineNode[];
  title: string;
  activeNodeId: string | null;
  onSelect: (id: string) => void;
  onAddRoot: (x?: number, y?: number) => void;
  onAddChild: (parentId: string, x?: number, y?: number) => void;
  onDelete: (id: string) => void;
  onRename: (id: string, title: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  onRenameTitle: (title: string) => void;
  onExport: () => void;
}

export default function OutlineTree({
  nodes,
  title,
  activeNodeId,
  onSelect,
  onAddRoot,
  onAddChild,
  onDelete,
  onRename,
  onMove,
  onRenameTitle,
  onExport,
}: OutlineTreeProps) {
  const layout = useMemo(() => layoutTree(nodes), [nodes]);
  const posMap = useMemo(() => {
    const m = new Map<string, { x: number; y: number }>();
    layout.forEach((l) => m.set(l.id, l));
    return m;
  }, [layout]);

  const drag = useRef<{ id: string; sx: number; sy: number; ox: number; oy: number; moved: boolean } | null>(null);
  const [dragPos, setDragPos] = useState<{ id: string; x: number; y: number } | null>(null);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const canvasRef = useRef<HTMLDivElement>(null);
  const [titleDraft, setTitleDraft] = useState(title);
  useEffect(() => setTitleDraft(title), [title]);

  function nodePos(id: string): { x: number; y: number } {
    if (dragPos && dragPos.id === id) return { x: dragPos.x, y: dragPos.y };
    return posMap.get(id) ?? { x: 0, y: 0 };
  }

  const edges: { fx: number; fy: number; tx: number; ty: number }[] = [];
  const visit = (n: OutlineNode) => {
    const p = nodePos(n.id);
    for (const c of n.children) {
      const cp = nodePos(c.id);
      edges.push({ fx: p.x, fy: p.y + NODE_H / 2, tx: cp.x, ty: cp.y - NODE_H / 2 });
      visit(c);
    }
  };
  nodes.forEach(visit);

  function startDrag(e: React.MouseEvent, id: string) {
    if (e.button !== 0) return;
    const p = posMap.get(id);
    if (!p) return;
    drag.current = { id, sx: e.clientX, sy: e.clientY, ox: p.x, oy: p.y, moved: false };

    const handleMove = (ev: MouseEvent) => {
      const d = drag.current;
      if (!d) return;
      const dx = ev.clientX - d.sx;
      const dy = ev.clientY - d.sy;
      if (!d.moved && Math.abs(dx) + Math.abs(dy) > 3) d.moved = true;
      if (d.moved) setDragPos({ id: d.id, x: d.ox + dx, y: d.oy + dy });
    };
    const handleUp = (ev: MouseEvent) => {
      const d = drag.current;
      drag.current = null;
      setDragPos(null);
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
      if (d && d.moved) {
        onMove(d.id, d.ox + (ev.clientX - d.sx), d.oy + (ev.clientY - d.sy));
      } else if (d) {
        onSelect(d.id);
      }
    };
    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
  }

  function handleCanvasDown(e: React.MouseEvent) {
    if ((e.target as HTMLElement).closest(".node-card")) return;
    const sx = e.clientX;
    const sy = e.clientY;
    const ox = pan.x;
    const oy = pan.y;
    const move = (ev: MouseEvent) => setPan({ x: ox + (ev.clientX - sx), y: oy + (ev.clientY - sy) });
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  // 滚轮/触控板捏合缩放（自然方向 + 迟钝），preventDefault 阻止浏览器原生页面缩放。
  useEffect(() => {
    const el = canvasRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      setZoom((z) => Math.max(0.5, Math.min(2.5, z * (1 - e.deltaY * 0.01))));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  // 当前可视区中心（世界坐标），新建节点落在这里，并加一点随机偏移避免完全重合。
  function worldCenter(): { x: number; y: number } | undefined {
    const el = canvasRef.current;
    if (!el) return undefined;
    const r = el.getBoundingClientRect();
    const jitter = () => (Math.random() - 0.5) * 30;
    return { x: (r.width / 2 - pan.x) / zoom + jitter(), y: (r.height / 2 - pan.y) / zoom + jitter() };
  }

  return (
    <div className="outline-inner">
      <div className="outline-head">
        <span className={`outline-view-label${titleDraft ? " on" : ""}`}>视图：</span>
        <input
          className="outline-title-input"
          value={titleDraft}
          onChange={(e) => setTitleDraft(e.target.value)}
          onBlur={() => onRenameTitle(titleDraft)}
          onKeyDown={(e) => {
            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
          }}
        />
        <button className="node-btn" title="一键导出" onClick={onExport}>
          ⤓
        </button>
        <button
          className="node-btn"
          title="添加根节点"
          onClick={() => {
            const c = worldCenter();
            onAddRoot(c?.x, c?.y);
          }}
        >
          ＋
        </button>
      </div>
      <div className="tree-canvas" ref={canvasRef} onMouseDown={handleCanvasDown}>
        {nodes.length === 0 ? (
          <div className="tree-empty">这个视图还没有节点，点右上角 ＋ 添加</div>
        ) : (
          <div className="tree-content" style={{ transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})` }}>
            <svg className="tree-edges">
              {edges.map((e, i) => (
                <path
                  key={i}
                  className="tree-edge"
                  d={`M ${e.fx} ${e.fy} C ${e.fx} ${e.fy + 26}, ${e.tx} ${e.ty - 26}, ${e.tx} ${e.ty}`}
                />
              ))}
            </svg>
            {flattenNodes(nodes).map((n) => {
              const pos = nodePos(n.id);
              return (
                <NodeCard
                  key={n.id}
                  node={n}
                  pos={pos}
                  active={activeNodeId === n.id}
                  onMouseDown={(e) => startDrag(e, n.id)}
                  onAddChild={(pid) => {
                    const c = worldCenter();
                    onAddChild(pid, c?.x, c?.y);
                  }}
                  onDelete={onDelete}
                  onRename={onRename}
                />
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function NodeCard({
  node,
  pos,
  active,
  onMouseDown,
  onAddChild,
  onDelete,
  onRename,
}: {
  node: OutlineNode;
  pos: { x: number; y: number };
  active: boolean;
  onMouseDown: (e: React.MouseEvent) => void;
  onAddChild: (id: string) => void;
  onDelete: (id: string) => void;
  onRename: (id: string, title: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(node.title);

  function commit() {
    onRename(node.id, draft.trim());
    setEditing(false);
  }

  return (
    <div className="node-wrap" style={{ left: pos.x, top: pos.y }}>
      <div
        className={`node-card${active ? " active" : ""}`}
        onMouseDown={onMouseDown}
        onDoubleClick={(e) => {
          e.stopPropagation();
          setDraft(node.title);
          setEditing(true);
        }}
      >
        {editing ? (
          <input
            className="node-rename"
            autoFocus
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => {
              if (e.key === "Enter") commit();
              if (e.key === "Escape") setEditing(false);
            }}
            onMouseDown={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="node-title">{node.title || "（未命名）"}</span>
        )}
      </div>
      <span className="node-actions">
        <button
          className="node-btn"
          title="加子节点"
          tabIndex={-1}
          onMouseDown={(e) => e.stopPropagation()}
          onClick={(e) => {
            e.stopPropagation();
            onAddChild(node.id);
          }}
        >
          ＋
        </button>
        <button
          className="node-btn node-del"
          title="删除"
          tabIndex={-1}
          onMouseDown={(e) => e.stopPropagation()}
          onClick={(e) => {
            e.stopPropagation();
            onDelete(node.id);
          }}
        >
          ×
        </button>
      </span>
    </div>
  );
}
