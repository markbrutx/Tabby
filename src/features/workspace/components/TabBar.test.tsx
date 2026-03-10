import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TabBar } from "./TabBar";

const tabs = [
  { id: "tab-1", title: "Workspace 1" },
  { id: "tab-2", title: "Workspace 2" },
];

function renderTabBar(overrides: Partial<Parameters<typeof TabBar>[0]> = {}) {
  const onSelect = vi.fn();
  const onClose = vi.fn();
  const onRename = vi.fn();
  const onNewTab = vi.fn();
  const onOpenSettings = vi.fn();
  const onOpenShortcuts = vi.fn();

  render(
    <TabBar
      tabs={tabs}
      activeTabId="tab-1"
      onSelect={onSelect}
      onClose={onClose}
      onRename={onRename}
      onNewTab={onNewTab}
      onOpenSettings={onOpenSettings}
      onOpenShortcuts={onOpenShortcuts}
      {...overrides}
    />,
  );

  return { onSelect, onClose, onRename, onNewTab, onOpenSettings, onOpenShortcuts };
}

describe("TabBar", () => {
  it("renders tab titles", () => {
    renderTabBar();
    expect(screen.getByText("Workspace 1")).toBeTruthy();
    expect(screen.getByText("Workspace 2")).toBeTruthy();
  });

  it("calls onSelect when tab is clicked", () => {
    const { onSelect } = renderTabBar();
    fireEvent.click(screen.getByTestId("tab-2"));
    expect(onSelect).toHaveBeenCalledWith("tab-2");
  });

  it("enters edit mode on double-click", () => {
    renderTabBar();
    fireEvent.doubleClick(screen.getByTestId("tab-1"));
    const input = screen.getByTestId("tab-rename-input-1");
    expect(input).toBeTruthy();
    expect((input as HTMLInputElement).value).toBe("Workspace 1");
  });

  it("commits rename on Enter", () => {
    const { onRename } = renderTabBar();
    fireEvent.doubleClick(screen.getByTestId("tab-1"));
    const input = screen.getByTestId("tab-rename-input-1");
    fireEvent.change(input, { target: { value: "My Tab" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onRename).toHaveBeenCalledWith("tab-1", "My Tab");
  });

  it("cancels rename on Escape", () => {
    const { onRename } = renderTabBar();
    fireEvent.doubleClick(screen.getByTestId("tab-1"));
    const input = screen.getByTestId("tab-rename-input-1");
    fireEvent.change(input, { target: { value: "My Tab" } });
    fireEvent.keyDown(input, { key: "Escape" });
    expect(onRename).not.toHaveBeenCalled();
    expect(screen.getByText("Workspace 1")).toBeTruthy();
  });

  it("commits rename on blur", () => {
    const { onRename } = renderTabBar();
    fireEvent.doubleClick(screen.getByTestId("tab-1"));
    const input = screen.getByTestId("tab-rename-input-1");
    fireEvent.change(input, { target: { value: "Blurred Tab" } });
    fireEvent.blur(input);
    expect(onRename).toHaveBeenCalledWith("tab-1", "Blurred Tab");
  });

  it("does not commit rename when input is empty", () => {
    const { onRename } = renderTabBar();
    fireEvent.doubleClick(screen.getByTestId("tab-1"));
    const input = screen.getByTestId("tab-rename-input-1");
    fireEvent.change(input, { target: { value: "" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onRename).not.toHaveBeenCalled();
  });
});
