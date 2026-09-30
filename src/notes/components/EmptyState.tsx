import type { View } from "../types";

export default function EmptyState({ view }: { view: View }) {
  return (
    <div className="empty-state">
      {view === "trash" ? "回收站是空的" : "还没有便签，点 ＋ 新建一条"}
    </div>
  );
}
