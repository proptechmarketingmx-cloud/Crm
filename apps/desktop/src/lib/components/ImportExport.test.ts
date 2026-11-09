// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const {
  importJsonWithMappingMock,
  listCustomFieldDefinitionsMock,
  openDialogMock,
  preflightJsonWithMappingMock,
  previewJsonMock,
  rollbackCompletedImportMock,
  restoreLocalBackupToAppDataMock,
  validateLocalBackupMock,
  reloadWorkspaceAfterDataReplaceMock,
} = vi.hoisted(
  () => ({
    importJsonWithMappingMock: vi.fn(),
    listCustomFieldDefinitionsMock: vi.fn(),
    openDialogMock: vi.fn(),
    preflightJsonWithMappingMock: vi.fn(),
    previewJsonMock: vi.fn(),
    rollbackCompletedImportMock: vi.fn(),
    restoreLocalBackupToAppDataMock: vi.fn(),
    validateLocalBackupMock: vi.fn(),
    reloadWorkspaceAfterDataReplaceMock: vi.fn(),
  }),
);

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: openDialogMock,
  save: vi.fn(),
}));

vi.mock("$lib/api/importExport", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/importExport")>();

  return {
    ...actual,
    importJsonWithMapping: importJsonWithMappingMock,
    preflightJsonWithMapping: preflightJsonWithMappingMock,
    previewJson: previewJsonMock,
    rollbackCompletedImport: rollbackCompletedImportMock,
  };
});

vi.mock("$lib/api/backup", () => ({
  restoreLocalBackupToAppData: restoreLocalBackupToAppDataMock,
  validateLocalBackup: validateLocalBackupMock,
}));

vi.mock("$lib/api/customFields", () => ({
  listCustomFieldDefinitions: listCustomFieldDefinitionsMock,
}));

vi.mock("$lib/stores/reloadWorkspace", () => ({
  reloadWorkspaceAfterDataReplace: reloadWorkspaceAfterDataReplaceMock,
}));

import ImportExport from "./ImportExport.svelte";

const backupPath = "/tmp/app-data/pre-import-backups/backup-1";
const backupValidation = {
  backup_dir: backupPath,
  database_path: `${backupPath}/900crm.db`,
  metadata_path: `${backupPath}/metadata.json`,
  metadata: {
    backup_format_version: 1,
    created_at: "2026-06-25T00:00:00Z",
    app_version: "0.1.0",
    schema_version: 1,
    device_id: "device-1",
    database_file: "900crm.db",
  },
};
const importWithBackupResult = {
  import: { created: 1, skipped: 0, errors: [] },
  backup: backupValidation,
};
const rollbackPlan = {
  token: "rollback-token-1",
  actions: [
    {
      entity_type: "contact",
      row_number: 2,
      entity_id: "contact-1",
      operation: "created",
      changed_fields: [],
      before_import: null,
      post_import: {
        contact_type: "person",
        first_name: "Ada",
        last_name: "",
        org_name: "",
        email: "ada@example.com",
        phone: "",
        address: "",
        city: "",
        country: "",
        org_id: null,
        organization_id: null,
        notes: "",
        updated_at: "2026-06-25T00:00:00Z",
      },
    },
  ],
};
const importWithRollbackResult = {
  import: { created: 1, skipped: 0, errors: [], rollback_plan: rollbackPlan },
  backup: backupValidation,
};

async function renderJsonImportSummary(result = importWithBackupResult) {
  openDialogMock.mockResolvedValue("/tmp/contacts.json");
  previewJsonMock.mockResolvedValue({
    total_rows: 1,
    headers: ["first_name", "email"],
    rows: [
      {
        row_number: 2,
        values: { first_name: "Ada", email: "ada@example.com" },
      },
    ],
  });
  preflightJsonWithMappingMock.mockResolvedValue({
    entity_type: "contacts",
    total_rows: 1,
    duplicate_warning_count: 1,
    warnings: [
      {
        entity_type: "contacts",
        row_number: 2,
        match_type: "email",
        csv_value: "ada@example.com",
        existing_entity_type: "contact",
        existing_entity_id: "contact-1",
        existing_display_label: "Ada Lovelace",
        reason: "Email 'ada@example.com' matches existing contact",
      },
    ],
  });
  importJsonWithMappingMock.mockResolvedValue(result);

  render(ImportExport, { open: true });

  await fireEvent.change(screen.getByLabelText("Format"), {
    target: { value: "json" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
  await waitFor(() => {
    expect(previewJsonMock).toHaveBeenCalledWith(
      "contacts",
      "/tmp/contacts.json",
    );
  });
  await screen.findByText("Ada");
  await fireEvent.click(screen.getByRole("button", { name: "Next" }));
  await screen.findByText("Column Mapping");
  await fireEvent.click(await screen.findByRole("button", { name: "Detect duplicates" }));
  await waitFor(() => {
    expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
      "contacts",
      "/tmp/contacts.json",
      {
        email: "email",
        first_name: "first_name",
      },
    );
  });
  await fireEvent.click(await screen.findByRole("button", { name: "Continue" }));
  await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));

  await waitFor(() => {
    expect(importJsonWithMappingMock).toHaveBeenCalledWith(
      "contacts",
      "/tmp/contacts.json",
      {
        email: "email",
        first_name: "first_name",
      },
    );
  });
  await screen.findByText("Created");
}

describe("ImportExport component", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    importJsonWithMappingMock.mockReset();
    listCustomFieldDefinitionsMock.mockReset();
    listCustomFieldDefinitionsMock.mockResolvedValue([]);
    openDialogMock.mockReset();
    preflightJsonWithMappingMock.mockReset();
    previewJsonMock.mockReset();
    rollbackCompletedImportMock.mockReset();
    restoreLocalBackupToAppDataMock.mockReset();
    validateLocalBackupMock.mockReset();
    reloadWorkspaceAfterDataReplaceMock.mockReset();
    reloadWorkspaceAfterDataReplaceMock.mockResolvedValue(undefined);
  });

  it("shows audit log as export-only entity", async () => {
    const { container } = render(ImportExport, { open: true });

    const importSelect = container.querySelector("#import-entity");
    expect(importSelect?.textContent).not.toContain("Audit log");

    await fireEvent.click(screen.getByRole("button", { name: "Export" }));

    const exportSelect = container.querySelector("#export-entity");
    expect(exportSelect?.textContent).toContain("Audit log");
  });

  it("shows proposed actions as export-only and disabled for import", async () => {
    const { container } = render(ImportExport, { open: true });

    const importOption = container.querySelector(
      '#import-entity option[disabled]',
    ) as HTMLOptionElement | null;
    expect(importOption?.textContent).toContain("Proposed actions (export only)");
    expect(importOption?.disabled).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: "Export" }));

    const exportSelect = container.querySelector("#export-entity");
    expect(exportSelect?.textContent).toContain("Proposed actions");
    expect(container.querySelector('#export-entity option[value="proposed_actions"]')).not.toBeNull();
  });

  it("shows external clients as export-only and disabled for import", async () => {
    const { container } = render(ImportExport, { open: true });

    const importOptions = Array.from(
      container.querySelectorAll("#import-entity option[disabled]"),
    ) as HTMLOptionElement[];
    const externalClientImportOption = importOptions.find((option) =>
      option.textContent?.includes("External clients (export only)"),
    );
    expect(externalClientImportOption).toBeTruthy();
    expect(externalClientImportOption?.disabled).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: "Export" }));

    const exportSelect = container.querySelector("#export-entity");
    expect(exportSelect?.textContent).toContain("External clients");
    expect(container.querySelector('#export-entity option[value="external_clients"]')).not.toBeNull();
  });

  it("shows external client permissions as export-only and disabled for import", async () => {
    const { container } = render(ImportExport, { open: true });

    const importOptions = Array.from(
      container.querySelectorAll("#import-entity option[disabled]"),
    ) as HTMLOptionElement[];
    const permissionImportOption = importOptions.find((option) =>
      option.textContent?.includes("External client permissions (export only)"),
    );
    expect(permissionImportOption).toBeTruthy();
    expect(permissionImportOption?.disabled).toBe(true);
    expect(
      container.querySelector('#import-entity option[value="external_client_permissions"]'),
    ).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Export" }));

    const exportSelect = container.querySelector("#export-entity");
    expect(exportSelect?.textContent).toContain("External client permissions");
    expect(
      container.querySelector('#export-entity option[value="external_client_permissions"]'),
    ).not.toBeNull();
  });

  it("previews JSON imports before duplicate preflight and confirmation", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["first_name", "email"],
      rows: [
        {
          row_number: 2,
          values: { first_name: "Ada", email: "ada@example.com" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: "contacts",
          row_number: 2,
          match_type: "email",
          csv_value: "ada@example.com",
          existing_entity_type: "contact",
          existing_entity_id: "contact-1",
          existing_display_label: "Ada Lovelace",
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(previewJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });
    expect(preflightJsonWithMappingMock).not.toHaveBeenCalled();
    expect(importJsonWithMappingMock).not.toHaveBeenCalled();
    expect(await screen.findByText("Ada")).toBeTruthy();
    expect(screen.getByText("ada@example.com")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText("Column Mapping")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: first_name")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: email")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));

    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
        {
          email: "email",
          first_name: "first_name",
        },
      );
    });

    expect(importJsonWithMappingMock).not.toHaveBeenCalled();
    expect(screen.getByText("1 duplicate warnings")).toBeTruthy();
    expect(screen.getByText("Ada Lovelace")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(importJsonWithMappingMock).not.toHaveBeenCalled();

    await fireEvent.click(
      screen.getByRole("button", { name: "Confirm import" }),
    );

    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
        {
          email: "email",
          first_name: "first_name",
        },
      );
    });

    expect(
      screen.getByText(
        `Automatic pre-import backup created at ${backupPath}`,
      ),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Restore pre-import backup" })).toBeTruthy();
  });

  it("skips duplicate review for activity JSON imports while keeping mapping and confirmation", async () => {
    listCustomFieldDefinitionsMock.mockResolvedValue([
      {
        id: "field-outcome",
        entity_type: "activity",
        field_name: "Outcome",
        field_type: "text",
        field_options: null,
        sort_order: 0,
        created_at: "2026-06-25T00:00:00Z",
      },
    ]);
    openDialogMock.mockResolvedValue("/tmp/activities.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["Kind", "Subject", "Outcome"],
      rows: [
        {
          row_number: 2,
          values: { Kind: "email", Subject: "Send update", Outcome: "Sent" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "activities",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "activities" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(listCustomFieldDefinitionsMock).toHaveBeenCalledWith("activity");
      expect(previewJsonMock).toHaveBeenCalledWith(
        "activities",
        "/tmp/activities.json",
      );
    });
    await screen.findByText("Send update");

    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText("Column Mapping")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Kind")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Subject")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Outcome")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Review import" }));

    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "activities",
        "/tmp/activities.json",
        {
          Kind: "activity_type",
          Outcome: "custom:Outcome",
          Subject: "title",
        },
      );
    });
    expect(screen.queryByText("Duplicate auto-merge is enabled for this import.")).toBeNull();
    expect(screen.getByText("Activity imports do not run duplicate detection. Confirmed rows create new activities.")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "activities",
        "/tmp/activities.json",
        {
          Kind: "activity_type",
          Outcome: "custom:Outcome",
          Subject: "title",
        },
      );
    });
    await screen.findByText("Created");
  });

  it("supports notes JSON imports through mapping and confirmation without duplicate review", async () => {
    openDialogMock.mockResolvedValue("/tmp/notes.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["Kind", "Target", "Body"],
      rows: [
        {
          row_number: 2,
          values: {
            Kind: "organization",
            Target: "organization-1",
            Body: "Imported organization note",
          },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "notes",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "notes" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(listCustomFieldDefinitionsMock).not.toHaveBeenCalled();
      expect(previewJsonMock).toHaveBeenCalledWith("notes", "/tmp/notes.json");
    });
    await screen.findByText("Imported organization note");

    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText("Column Mapping")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Kind")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Target")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Body")).toBeTruthy();
    expect(screen.queryByLabelText("Merge duplicate rows into existing records")).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Review import" }));

    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "notes",
        "/tmp/notes.json",
        {
          Body: "content",
          Kind: "entity_type",
          Target: "entity_id",
        },
      );
    });
    expect(screen.getByText("Note imports do not run duplicate detection. Confirmed rows create new notes.")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "notes",
        "/tmp/notes.json",
        {
          Body: "content",
          Kind: "entity_type",
          Target: "entity_id",
        },
      );
    });
    await screen.findByText("Created");
  });

  it("supports tag link JSON imports through local ID mapping without duplicate review", async () => {
    openDialogMock.mockResolvedValue("/tmp/tag-links.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["Parent Type", "Parent ID", "Local Tag ID"],
      rows: [
        {
          row_number: 2,
          values: {
            "Parent Type": "contact",
            "Parent ID": "contact-1",
            "Local Tag ID": "tag-1",
          },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "tag_links",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "tag_links" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(listCustomFieldDefinitionsMock).not.toHaveBeenCalled();
      expect(previewJsonMock).toHaveBeenCalledWith("tag_links", "/tmp/tag-links.json");
    });
    await screen.findByText("tag-1");

    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByLabelText("Map to field: Parent Type")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Parent ID")).toBeTruthy();
    expect(screen.getByLabelText("Map to field: Local Tag ID")).toBeTruthy();
    expect(screen.queryByLabelText("Merge duplicate rows into existing records")).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Review import" }));

    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "tag_links",
        "/tmp/tag-links.json",
        {
          "Local Tag ID": "tag_id",
          "Parent ID": "entity_id",
          "Parent Type": "entity_type",
        },
      );
    });
    expect(screen.getByText("Tag link imports do not run duplicate detection. Existing active local links are skipped.")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "tag_links",
        "/tmp/tag-links.json",
        {
          "Local Tag ID": "tag_id",
          "Parent ID": "entity_id",
          "Parent Type": "entity_type",
        },
      );
    });
    await screen.findByText("Created");
  });

  it("maps nonstandard JSON source fields before preflight and import", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts-custom.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["Given", "Mail"],
      rows: [
        {
          row_number: 2,
          values: { Given: "Ada", Mail: "ada@example.com" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Ada");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));

    await fireEvent.change(screen.getByLabelText("Map to field: Given"), {
      target: { value: "first_name" },
    });
    await fireEvent.change(screen.getByLabelText("Map to field: Mail"), {
      target: { value: "email" },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));
    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts-custom.json",
        {
          Given: "first_name",
          Mail: "email",
        },
      );
    });

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts-custom.json",
        {
          Given: "first_name",
          Mail: "email",
        },
      );
    });
  });

  it("includes contact custom field targets in JSON mapping payloads", async () => {
    listCustomFieldDefinitionsMock.mockResolvedValue([
      {
        id: "field-vip-tier",
        entity_type: "contact",
        field_name: "VIP Tier",
        field_type: "text",
        field_options: null,
        sort_order: 0,
        created_at: "2026-06-25T00:00:00Z",
      },
    ]);
    openDialogMock.mockResolvedValue("/tmp/contacts-custom-fields.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["first_name", "VIP Tier"],
      rows: [
        {
          row_number: 2,
          values: { first_name: "Ada", "VIP Tier": "Gold" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Gold");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));

    expect(screen.getByLabelText("Map to field: VIP Tier")).toBeTruthy();
    expect(screen.getAllByRole("option", { name: "Custom: VIP Tier" }).length).toBeGreaterThan(0);

    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));
    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts-custom-fields.json",
        {
          "VIP Tier": "custom:VIP Tier",
          first_name: "first_name",
        },
      );
    });

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts-custom-fields.json",
        {
          "VIP Tier": "custom:VIP Tier",
          first_name: "first_name",
        },
      );
    });
  });

  it("includes organization custom field targets in JSON mapping payloads", async () => {
    listCustomFieldDefinitionsMock.mockResolvedValue([
      {
        id: "field-segment",
        entity_type: "organization",
        field_name: "Segment",
        field_type: "text",
        field_options: null,
        sort_order: 0,
        created_at: "2026-06-25T00:00:00Z",
      },
    ]);
    openDialogMock.mockResolvedValue("/tmp/organizations-custom-fields.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["Company", "Segment"],
      rows: [
        {
          row_number: 2,
          values: { Company: "Acme Health", Segment: "Enterprise" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "organizations",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    importJsonWithMappingMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "organizations" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await waitFor(() => {
      expect(listCustomFieldDefinitionsMock).toHaveBeenCalledWith("organization");
    });
    await screen.findByText("Enterprise");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));

    expect(screen.getByLabelText("Map to field: Segment")).toBeTruthy();
    expect(screen.getAllByRole("option", { name: "Custom: Segment" }).length).toBeGreaterThan(0);

    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));
    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "organizations",
        "/tmp/organizations-custom-fields.json",
        {
          Company: "name",
          Segment: "custom:Segment",
        },
      );
    });

    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));
    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "organizations",
        "/tmp/organizations-custom-fields.json",
        {
          Company: "name",
          Segment: "custom:Segment",
        },
      );
    });
  });

  it("blocks duplicate JSON field mappings before preflight or import", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts-duplicate-mapping.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["First Name", "Given Name", "Email"],
      rows: [
        {
          row_number: 2,
          values: {
            "First Name": "Ada",
            "Given Name": "Augusta",
            Email: "ada@example.com",
          },
        },
      ],
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Ada");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "First name is mapped more than once: First Name, Given Name.",
    );
    expect(preflightJsonWithMappingMock).not.toHaveBeenCalled();
    expect(importJsonWithMappingMock).not.toHaveBeenCalled();
  });

  it("does not allow JSON duplicate preflight while preview is loading", async () => {
    let resolvePreview: (preview: {
      total_rows: number;
      headers: string[];
      rows: Array<{ row_number: number; values: Record<string, string> }>;
    }) => void = () => {};

    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    previewJsonMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolvePreview = resolve;
        }),
    );
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(previewJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });

    const nextButton = screen.getByRole("button", { name: "Next" }) as HTMLButtonElement;
    expect(nextButton.disabled).toBe(true);
    await fireEvent.click(nextButton);
    expect(preflightJsonWithMappingMock).not.toHaveBeenCalled();
    expect(importJsonWithMappingMock).not.toHaveBeenCalled();

    resolvePreview({
      total_rows: 1,
      headers: ["first_name"],
      rows: [{ row_number: 2, values: { first_name: "Ada" } }],
    });

    await screen.findByText("Ada");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await screen.findByText("Column Mapping");
    const readyDetectButton = screen.getByRole("button", { name: "Detect duplicates" }) as HTMLButtonElement;
    expect(readyDetectButton.disabled).toBe(false);

    await fireEvent.click(readyDetectButton);
    await waitFor(() => {
      expect(preflightJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
        {
          first_name: "first_name",
        },
      );
    });
  });

  it("keeps invalid JSON preview errors from running preflight or import", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts-invalid.json");
    previewJsonMock.mockRejectedValue(new Error("JSON import expects a top-level array"));

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "JSON preview failed. Choose a valid JSON array of supported rows.",
    );
    expect(preflightJsonWithMappingMock).not.toHaveBeenCalled();
    expect(importJsonWithMappingMock).not.toHaveBeenCalled();
    expect((screen.getByRole("button", { name: "Next" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows duplicate auto-merge for contact, deal, and organization imports", async () => {
    openDialogMock.mockResolvedValue("/tmp/import.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["name", "email", "title"],
      rows: [
        {
          row_number: 2,
          values: { name: "Acme", email: "hello@acme.test", title: "Renewal" },
        },
      ],
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("hello@acme.test");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByLabelText("Merge duplicate rows into existing records")).toBeTruthy();

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "organizations" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("hello@acme.test");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByLabelText("Merge duplicate rows into existing records")).toBeTruthy();

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "deals" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Renewal");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByLabelText("Merge duplicate rows into existing records")).toBeTruthy();
  });

  it("sends enabled duplicate auto-merge through import and shows merged summary", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["first_name", "email"],
      rows: [
        {
          row_number: 2,
          values: { first_name: "Ada", email: "ada@example.com" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: "contacts",
          row_number: 2,
          match_type: "email",
          csv_value: "ada@example.com",
          existing_entity_type: "contact",
          existing_entity_id: "contact-1",
          existing_display_label: "Ada Lovelace",
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    importJsonWithMappingMock.mockResolvedValue({
      import: { created: 0, merged: 1, skipped: 0, errors: [] },
      backup: backupValidation,
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Ada");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await fireEvent.click(screen.getByLabelText("Merge duplicate rows into existing records"));
    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));
    await screen.findByText("Duplicate auto-merge is enabled for this import.");
    await fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(
      screen.getByText(
        "Duplicate auto-merge is enabled. Safe contact, deal, or organization matches will merge instead of creating duplicate records.",
      ),
    ).toBeTruthy();
    expect(
      screen.getByText("1 duplicate warnings will be merged into matching existing records where safe."),
    ).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));

    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
        {
          email: "email",
          first_name: "first_name",
        },
        { mergeDuplicates: true },
      );
    });
    const mergedLabel = await screen.findByText("Merged");
    expect(mergedLabel.nextElementSibling?.textContent).toBe("1");
  });

  it("sends enabled duplicate auto-merge through deal import", async () => {
    openDialogMock.mockResolvedValue("/tmp/deals.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["title", "value"],
      rows: [
        {
          row_number: 2,
          values: { title: "Acme Renewal", value: "7500" },
        },
      ],
    });
    preflightJsonWithMappingMock.mockResolvedValue({
      entity_type: "deals",
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: "deals",
          row_number: 2,
          match_type: "title",
          csv_value: "Acme Renewal",
          existing_entity_type: "deal",
          existing_entity_id: "deal-1",
          existing_display_label: "Acme Renewal",
          reason: "Title 'Acme Renewal' matches existing deal",
        },
      ],
    });
    importJsonWithMappingMock.mockResolvedValue({
      import: { created: 0, merged: 1, skipped: 0, errors: [] },
      backup: backupValidation,
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Type"), {
      target: { value: "deals" },
    });
    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
    await screen.findByText("Acme Renewal");
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await fireEvent.click(screen.getByLabelText("Merge duplicate rows into existing records"));
    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));
    await screen.findByText("Duplicate auto-merge is enabled for this import.");
    await fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));

    await waitFor(() => {
      expect(importJsonWithMappingMock).toHaveBeenCalledWith(
        "deals",
        "/tmp/deals.json",
        {
          title: "title",
          value: "value",
        },
        { mergeDuplicates: true },
      );
    });
    const mergedLabel = await screen.findByText("Merged");
    expect(mergedLabel.nextElementSibling?.textContent).toBe("1");
  });

  it("does not show the restore control when the import summary has no backup path", async () => {
    await renderJsonImportSummary({
      import: { created: 1, skipped: 0, errors: [] },
      backup: { ...backupValidation, backup_dir: "" },
    });

    expect(screen.queryByRole("button", { name: "Restore pre-import backup" })).toBeNull();
  });

  it("rolls back rows from the current import summary after explicit confirmation", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    rollbackCompletedImportMock.mockResolvedValue({
      token: "rollback-token-1",
      rolled_back: 1,
      skipped: 0,
      errors: [],
    });

    await renderJsonImportSummary(importWithRollbackResult);

    expect(screen.getByText("Row-level rollback is available for 1 imported rows.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Restore pre-import backup" })).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Rollback imported rows" }));

    await waitFor(() => {
      expect(rollbackCompletedImportMock).toHaveBeenCalledWith(rollbackPlan);
    });
    expect(
      screen.getByText("Row rollback complete: 1 rolled back, 0 skipped, 0 row errors."),
    ).toBeTruthy();
    expect((screen.getByRole("button", { name: "Rollback imported rows" }) as HTMLButtonElement).disabled).toBe(true);
    expect(validateLocalBackupMock).not.toHaveBeenCalled();
  });

  it("does not rollback imported rows when confirmation is cancelled", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(false);

    await renderJsonImportSummary(importWithRollbackResult);

    await fireEvent.click(screen.getByRole("button", { name: "Rollback imported rows" }));

    expect(rollbackCompletedImportMock).not.toHaveBeenCalled();
    expect(screen.getByText("Row rollback cancelled")).toBeTruthy();
  });

  it("does not restore the pre-import backup when confirmation is cancelled", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(false);
    validateLocalBackupMock.mockResolvedValue(backupValidation);

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(validateLocalBackupMock).toHaveBeenCalledWith(backupPath);
    });
    expect(restoreLocalBackupToAppDataMock).not.toHaveBeenCalled();
    expect(screen.getByText("Pre-import backup restore cancelled")).toBeTruthy();
  });

  it("validates then restores the pre-import backup after explicit confirmation", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    validateLocalBackupMock.mockResolvedValue(backupValidation);
    restoreLocalBackupToAppDataMock.mockResolvedValue({
      restored_at: "2026-06-25T01:00:00Z",
      database_path: "/app-data/900crm.db",
      metadata: backupValidation.metadata,
    });

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(validateLocalBackupMock).toHaveBeenCalledWith(backupPath);
    });
    expect(restoreLocalBackupToAppDataMock).toHaveBeenCalledWith(backupPath, true);
    expect(reloadWorkspaceAfterDataReplaceMock).toHaveBeenCalled();
    expect(screen.getByText("Pre-import backup restored to /app-data/900crm.db")).toBeTruthy();
  });

  it("keeps the import summary visible when pre-import backup restore fails", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    validateLocalBackupMock.mockResolvedValue(backupValidation);
    restoreLocalBackupToAppDataMock.mockRejectedValue(new Error("restore failed"));

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain(
        "Pre-import backup restore failed: restore failed",
      );
    });
    expect(screen.getByText(`Automatic pre-import backup created at ${backupPath}`)).toBeTruthy();
  });
});
