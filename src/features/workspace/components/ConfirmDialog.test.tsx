import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConfirmDialog } from "./ConfirmDialog";

function renderDialog(overrides: Partial<Parameters<typeof ConfirmDialog>[0]> = {}) {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();

  render(
    <ConfirmDialog
      title="Close pane?"
      message="The terminal session will be terminated."
      onConfirm={onConfirm}
      onCancel={onCancel}
      {...overrides}
    />,
  );

  return { onConfirm, onCancel };
}

describe("ConfirmDialog", () => {
  it("renders title and message", () => {
    renderDialog();
    expect(screen.getByText("Close pane?")).toBeTruthy();
    expect(screen.getByText("The terminal session will be terminated.")).toBeTruthy();
  });

  it("confirm button fires onConfirm", () => {
    const { onConfirm } = renderDialog();
    fireEvent.click(screen.getByTestId("confirm-ok"));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("cancel button fires onCancel", () => {
    const { onCancel } = renderDialog();
    fireEvent.click(screen.getByTestId("confirm-cancel"));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("Escape key fires onCancel", () => {
    const { onCancel } = renderDialog();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("Enter key fires onConfirm", () => {
    const { onConfirm } = renderDialog();
    fireEvent.keyDown(window, { key: "Enter" });
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("backdrop click fires onCancel", () => {
    const { onCancel } = renderDialog();
    fireEvent.click(screen.getByTestId("confirm-dialog"));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("uses custom labels", () => {
    renderDialog({ confirmLabel: "Delete", cancelLabel: "Keep" });
    expect(screen.getByText("Delete")).toBeTruthy();
    expect(screen.getByText("Keep")).toBeTruthy();
  });
});
