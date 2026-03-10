import { fireEvent, render, screen, within, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BranchSelector, type BranchSelectorProps } from "./BranchSelector";
import type { BranchInfo } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeBranch(overrides?: Partial<BranchInfo>): BranchInfo {
  return {
    name: "main",
    isCurrent: false,
    upstream: null,
    ahead: 0,
    behind: 0,
    ...overrides,
  };
}

function defaultBranches(): readonly BranchInfo[] {
  return [
    makeBranch({ name: "main", isCurrent: true, upstream: "origin/main", ahead: 1, behind: 0 }),
    makeBranch({ name: "feature/git-client", upstream: "origin/feature/git-client", ahead: 0, behind: 2 }),
    makeBranch({ name: "develop" }),
  ];
}

function renderSelector(overrides?: Partial<BranchSelectorProps>) {
  const defaults: BranchSelectorProps = {
    branches: defaultBranches(),
    loading: false,
    onCheckout: vi.fn().mockResolvedValue(undefined),
    onCreateBranch: vi.fn().mockResolvedValue(undefined),
    onDeleteBranch: vi.fn().mockResolvedValue(undefined),
    onRefresh: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
  return { ...render(<BranchSelector {...defaults} />), props: defaults };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("BranchSelector", () => {
  it("renders branch list with all branches", () => {
    renderSelector();
    const items = screen.getAllByTestId("branch-item");
    expect(items).toHaveLength(3);
  });

  it("highlights current branch", () => {
    renderSelector();
    const items = screen.getAllByTestId("branch-item");
    // First branch (main) is current — should have accent background class
    expect(items[0].className).toContain("accent");
    // Second branch is not current
    expect(items[1].className).not.toContain("accent");
  });

  it("displays current branch name in header", () => {
    renderSelector();
    expect(screen.getByTestId("current-branch-name")).toHaveTextContent("main");
  });

  it("shows ahead/behind counts for tracking branches", () => {
    renderSelector();
    const badges = screen.getAllByTestId("ahead-behind");
    // main: +1, feature/git-client: -2
    expect(badges).toHaveLength(2);
    expect(badges[0]).toHaveTextContent("+1");
    expect(badges[1]).toHaveTextContent("-2");
  });

  it("calls onCheckout when clicking a non-current branch", async () => {
    const { props } = renderSelector();
    const checkoutButtons = screen.getAllByTestId("branch-checkout-button");
    // Second button (feature/git-client) is not current
    fireEvent.click(checkoutButtons[1]);
    await waitFor(() => {
      expect(props.onCheckout).toHaveBeenCalledWith("feature/git-client");
    });
  });

  it("disables checkout for the current branch", () => {
    renderSelector();
    const checkoutButtons = screen.getAllByTestId("branch-checkout-button");
    expect(checkoutButtons[0]).toBeDisabled();
  });

  it("filters branches by search query", () => {
    renderSelector();
    fireEvent.change(screen.getByTestId("branch-search"), {
      target: { value: "feat" },
    });
    const items = screen.getAllByTestId("branch-item");
    expect(items).toHaveLength(1);
    expect(items[0]).toHaveTextContent("feature/git-client");
  });

  it("shows empty message when filter matches nothing", () => {
    renderSelector();
    fireEvent.change(screen.getByTestId("branch-search"), {
      target: { value: "nonexistent" },
    });
    expect(screen.getByTestId("branch-list-empty")).toHaveTextContent("No branches match your filter");
  });

  it("shows create branch form when clicking + button", () => {
    renderSelector();
    fireEvent.click(screen.getByTestId("create-branch-button"));
    expect(screen.getByTestId("create-branch-form")).toBeInTheDocument();
  });

  it("calls onCreateBranch with name and start point", async () => {
    const { props } = renderSelector();
    fireEvent.click(screen.getByTestId("create-branch-button"));

    fireEvent.change(screen.getByTestId("create-branch-name"), {
      target: { value: "feature/new" },
    });
    fireEvent.change(screen.getByTestId("create-branch-start-point"), {
      target: { value: "develop" },
    });
    fireEvent.click(screen.getByTestId("create-branch-submit"));

    await waitFor(() => {
      expect(props.onCreateBranch).toHaveBeenCalledWith("feature/new", "develop");
    });
  });

  it("calls onCreateBranch with null start point when left empty", async () => {
    const { props } = renderSelector();
    fireEvent.click(screen.getByTestId("create-branch-button"));

    fireEvent.change(screen.getByTestId("create-branch-name"), {
      target: { value: "hotfix/urgent" },
    });
    fireEvent.click(screen.getByTestId("create-branch-submit"));

    await waitFor(() => {
      expect(props.onCreateBranch).toHaveBeenCalledWith("hotfix/urgent", null);
    });
  });

  it("disables create submit when name is empty", () => {
    renderSelector();
    fireEvent.click(screen.getByTestId("create-branch-button"));
    expect(screen.getByTestId("create-branch-submit")).toBeDisabled();
  });

  it("shows delete confirmation when clicking delete button", () => {
    renderSelector();
    const deleteButtons = screen.getAllByTestId("branch-delete-button");
    fireEvent.click(deleteButtons[0]);
    expect(screen.getByTestId("delete-confirm")).toBeInTheDocument();
  });

  it("calls onDeleteBranch with force=false on normal delete", async () => {
    const { props } = renderSelector();
    const deleteButtons = screen.getAllByTestId("branch-delete-button");
    fireEvent.click(deleteButtons[0]);

    fireEvent.click(screen.getByTestId("delete-confirm-yes"));

    await waitFor(() => {
      expect(props.onDeleteBranch).toHaveBeenCalledWith("feature/git-client", false);
    });
  });

  it("calls onDeleteBranch with force=true on force delete", async () => {
    const { props } = renderSelector();
    const deleteButtons = screen.getAllByTestId("branch-delete-button");
    fireEvent.click(deleteButtons[0]);

    fireEvent.click(screen.getByTestId("delete-confirm-force"));

    await waitFor(() => {
      expect(props.onDeleteBranch).toHaveBeenCalledWith("feature/git-client", true);
    });
  });

  it("cancels delete confirmation", () => {
    renderSelector();
    const deleteButtons = screen.getAllByTestId("branch-delete-button");
    fireEvent.click(deleteButtons[0]);
    expect(screen.getByTestId("delete-confirm")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("delete-confirm-cancel"));
    expect(screen.queryByTestId("delete-confirm")).not.toBeInTheDocument();
  });

  it("does not show delete button for current branch", () => {
    renderSelector();
    const items = screen.getAllByTestId("branch-item");
    const currentItem = items[0]; // main is current
    expect(within(currentItem).queryByTestId("branch-delete-button")).not.toBeInTheDocument();
  });

  it("shows loading state", () => {
    renderSelector({ loading: true });
    expect(screen.getByTestId("branch-loading")).toBeInTheDocument();
  });

  it("shows upstream indicator for tracking branches", () => {
    renderSelector();
    const upstreams = screen.getAllByTestId("branch-upstream");
    expect(upstreams).toHaveLength(2); // main and feature/git-client have upstreams
  });

  it("calls onRefresh when clicking refresh button", () => {
    const { props } = renderSelector();
    fireEvent.click(screen.getByTestId("refresh-branches-button"));
    expect(props.onRefresh).toHaveBeenCalled();
  });
});
