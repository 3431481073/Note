export default function TrashConfirm({
  onConfirm,
  onCancel,
}: {
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="confirm-mask" onClick={onCancel} data-hit>
      <div className="confirm-box" onClick={(e) => e.stopPropagation()}>
        <div className="confirm-title">彻底删除？</div>
        <div className="confirm-desc">删除后不可恢复。</div>
        <div className="confirm-actions">
          <button className="confirm-cancel" onClick={onCancel}>
            取消
          </button>
          <button className="confirm-ok" onClick={onConfirm}>
            彻底删除
          </button>
        </div>
      </div>
    </div>
  );
}
