import type { ReactNode } from "react";

interface DrawerOverlayProps {
  side: "left" | "right";
  maxWidth: number;
  children: ReactNode;
  onClose: () => void;
}

export function DrawerOverlay({
  side,
  maxWidth,
  children,
  onClose,
}: DrawerOverlayProps) {
  const justify = side === "right" ? "justify-end" : "";
  const rounding =
    side === "left" ? "rounded-r-[28px]" : "rounded-l-[28px]";

  return (
    <div className={`fixed inset-0 z-50 flex bg-black/50 ${justify}`}>
      {side === "right" && (
        <button
          className="flex-1"
          aria-label="Close"
          onClick={onClose}
        />
      )}
      <div
        className={`surface-panel flex h-full w-full flex-col rounded-none ${rounding} p-5`}
        style={{ maxWidth }}
      >
        {children}
      </div>
      {side === "left" && (
        <button
          className="flex-1"
          aria-label="Close"
          onClick={onClose}
        />
      )}
    </div>
  );
}
