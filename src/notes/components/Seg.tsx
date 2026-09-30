import type { Mode } from "../types";

export default function Seg({ mode, onChange }: { mode: Mode; onChange: (m: Mode) => void }) {
  return (
    <div className="seg">
      <button
        className={`seg-btn${mode === "card" ? " on" : ""}`}
        onClick={() => onChange("card")}
      >
        <span className="seg-note">Note</span>
      </button>
      <button
        className={`seg-btn${mode === "outline" ? " on" : ""}`}
        onClick={() => onChange("outline")}
      >
        视图
      </button>
    </div>
  );
}
