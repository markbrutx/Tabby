import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { StashPanel, type StashPanelProps } from "./StashPanel";
import type { StashEntry } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeStash(overrides?: Partial<StashEntry>): StashEntry {
  return {
    index: 0,
    message: "WIP on main: work in progress",
    date: "2026-03-10T11:00:00Z",
    ...overrides,
  };
}

function defaultStashes(): readonly StashEntry[] {
  return [
    makeStash({ index: 0, message: "WIP on main: work in progress" }),
    makeStash({ index: 1, message: "WIP on feature: partial implementation", date: "2026-03-09T15:00:00Z" }),
    makeStash({ index: 2, message: "Saving before rebase", date: "2026-03-08T09:00:00Z" }),
  ];
}

function renderPanel(overrides?: Partial<StashPanelProps>) {
  const defaults: StashPanelProps = {
    stashes: defaultStashes(),
    loading: false,
    onPush: vi.fn(),
    onPop: vi.fn(),
    onApply: vi.fn(),
    onDrop: vi.fn(),
    onRefresh: vi.fn(),
    ...overrides,
  };
  return { ...render(<StashPanel {...defaults} />), props: defaults };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("StashPanel", () => {
  it("renders all stashes in the list", () => {
    renderPanel();
    expect(screen.getByTestId("stash-row-0")).toBeDefined();
    expect(screen.getByTestId("stash-row-1")).toBeDefined();
    expect(screen.getByTestId("stash-row-2")).toBeDefined();
  });

  it("displays stash index and message", () => {
    renderPanel();
    const row = screen.getByTestId("stash-row-0");
    expect(row.textContent).toContain("stash@{0}");
    expect(row.textContent).toContain("WIP on main: work in progress");
  });

  it("shows empty state when no stashes", () => {
    renderPanel({ stashes: [] });
    expect(screen.getByTestId("stash-empty")).toBeDefined();
    expect(screen.getByText("No stashes")).toBeDefined();
  });

  it("shows loading indicator when loading", () => {
    renderPanel({ loading: true });
    expect(screen.getByTestId("stash-loading")).toBeDefined();
  });

  it("calls onPush with message when push button is clicked", () => {
    const { props } = renderPanel();
    const input = screen.getByTestId("stash-message-input");
    fireEvent.change(input, { target: { value: "my stash message" } });
    fireEvent.click(screen.getByTestId("stash-push-button"));
    expect(props.onPush).toHaveBeenCalledWith("my stash message");
  });

  it("calls onPush with null when push button is clicked without message", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-push-button"));
    expect(props.onPush).toHaveBeenCalledWith(null);
  });

  it("calls onPush on Enter key in input", () => {
    const { props } = renderPanel();
    const input = screen.getByTestId("stash-message-input");
    fireEvent.change(input, { target: { value: "enter stash" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(props.onPush).toHaveBeenCalledWith("enter stash");
  });

  it("shows action bar when a stash is selected", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-1"));
    expect(screen.getByTestId("stash-actions")).toBeDefined();
    expect(screen.getByTestId("stash-pop-button")).toBeDefined();
    expect(screen.getByTestId("stash-apply-button")).toBeDefined();
    expect(screen.getByTestId("stash-drop-button")).toBeDefined();
  });

  it("calls onPop when pop button is clicked", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-1"));
    fireEvent.click(screen.getByTestId("stash-pop-button"));
    expect(props.onPop).toHaveBeenCalledWith(1);
  });

  it("calls onApply when apply button is clicked", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    fireEvent.click(screen.getByTestId("stash-apply-button"));
    expect(props.onApply).toHaveBeenCalledWith(0);
  });

  it("shows drop confirmation when drop button is clicked", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    fireEvent.click(screen.getByTestId("stash-drop-button"));
    expect(screen.getByTestId("stash-drop-confirm")).toBeDefined();
  });

  it("calls onDrop when drop is confirmed", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    fireEvent.click(screen.getByTestId("stash-drop-button"));
    fireEvent.click(screen.getByTestId("stash-drop-confirm-button"));
    expect(props.onDrop).toHaveBeenCalledWith(0);
  });

  it("cancels drop when cancel is clicked", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    fireEvent.click(screen.getByTestId("stash-drop-button"));
    fireEvent.click(screen.getByTestId("stash-drop-cancel-button"));
    expect(props.onDrop).not.toHaveBeenCalled();
    expect(screen.queryByTestId("stash-drop-confirm")).toBeNull();
  });

  it("deselects stash when clicking same row again", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    expect(screen.getByTestId("stash-actions")).toBeDefined();
    fireEvent.click(screen.getByTestId("stash-row-0"));
    expect(screen.queryByTestId("stash-actions")).toBeNull();
  });

  it("highlights selected stash row", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("stash-row-1"));
    const row = screen.getByTestId("stash-row-1");
    expect(row.className).toContain("accent");
  });

  it("calls onRefresh when refresh button is clicked", () => {
    const { props } = renderPanel();
    fireEvent.click(screen.getByTestId("stash-refresh-button"));
    expect(props.onRefresh).toHaveBeenCalled();
  });

  it("clears input after push", () => {
    renderPanel();
    const input = screen.getByTestId("stash-message-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "some message" } });
    fireEvent.click(screen.getByTestId("stash-push-button"));
    expect(input.value).toBe("");
  });
});
