import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { PageMeta, View } from "../types";
import Card from "./Card";

export interface CardListHandle {
  scrollToIndex: (
    index: number,
    align?: "auto" | "start" | "center" | "end",
  ) => void;
}

interface CardListProps {
  items: PageMeta[];
  view: View;
  activeId: string | null;
  hasMore: boolean;
  onLoadMore: () => void;
  onSelect: (id: string) => void;
  onDoubleSelect: (id: string) => void;
  onDelete: (id: string) => void;
  onRestore: (id: string) => void;
  onPurge: (id: string) => void;
}

const CardList = forwardRef<CardListHandle, CardListProps>((props, ref) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: props.items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 92,
    overscan: 3,
  });

  useImperativeHandle(
    ref,
    () => ({
      scrollToIndex: (index, align = "center") =>
        virtualizer.scrollToIndex(index, { align }),
    }),
    [virtualizer],
  );

  // 滚动中给滚动容器打标，暂停活跃卡片的光带动画以缓解滚动卡顿。
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    let timer: number | null = null;
    const onScroll = () => {
      el.dataset.scrolling = "true";
      if (timer !== null) window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        delete el.dataset.scrolling;
      }, 120);
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      el.removeEventListener("scroll", onScroll);
      if (timer !== null) window.clearTimeout(timer);
    };
  }, []);

  return (
    <div className="card-list-outer">
      <div className="card-list-scroll" ref={scrollRef}>
        <div
          className="card-list-inner"
          style={{ height: `${virtualizer.getTotalSize()}px` }}
        >
          {virtualizer.getVirtualItems().map((vi) => {
            const meta = props.items[vi.index];
            return (
              <div
                key={meta.id}
                className="card-slot"
                style={{ transform: `translateY(${vi.start}px)` }}
              >
                <Card
                  meta={meta}
                  active={meta.id === props.activeId}
                  view={props.view}
                  onSelect={props.onSelect}
                  onDoubleSelect={props.onDoubleSelect}
                  onDelete={props.onDelete}
                  onRestore={props.onRestore}
                  onPurge={props.onPurge}
                />
              </div>
            );
          })}
        </div>
        {props.hasMore && (
          <button className="load-more" onClick={props.onLoadMore}>
            加载更多
          </button>
        )}
      </div>
    </div>
  );
});
CardList.displayName = "CardList";
export default CardList;
