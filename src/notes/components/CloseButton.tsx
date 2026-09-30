export default function CloseButton({ onClose }: { onClose: () => void }) {
  return (
    <button className="close-btn" title="关闭" onClick={onClose} aria-label="关闭" data-hit>
      ×
    </button>
  );
}
