import { promises as fs } from "node:fs";
import path from "node:path";
import { parse } from "yaml";
import { describe, expect, test } from "vitest";

const repoRoot = path.resolve(process.cwd(), "..");
const specRoot = path.join(repoRoot, "docs/spec");

describe("executable documentation sources", () => {
  test("REQ-OPS-003: legacy requirement registries are retired", async () => {
    // Mitase owns requirement taxonomy now. This guard keeps the retired
    // docs/spec tree from returning as a second authority.
    await expect(fs.stat(specRoot)).rejects.toThrow();
  });

  test("REQ-OPS-004: version statuses agree with their tasks and canonical sources", async () => {
    const versionRoot = path.join(repoRoot, "docs/version");

    for (const filename of await yamlFilesRecursively(versionRoot)) {
      const filePath = path.join(versionRoot, filename);
      const document = parse(
        await fs.readFile(filePath, "utf8"),
      ) as VersionDocument;

      // Changelog entries are historical records, not status-bearing plans.
      if (document.status === undefined) continue;

      assertVersionStatus(document.status, filename);
      const phaseStatuses = [] as VersionStatus[];
      for (const phase of document.phases ?? []) {
        assertVersionStatus(phase.status, `${filename}:${phase.id}`);
        phaseStatuses.push(phase.status);
        expect(
          Array.isArray(phase.tasks),
          `${filename}:${phase.id}: status must be supported by tasks`,
        ).toBe(true);
        const tasks = phase.tasks ?? [];
        for (const [index, task] of tasks.entries()) {
          expect(
            typeof task.done,
            `${filename}:${phase.id}: task ${index} must declare done`,
          ).toBe("boolean");
        }
        expect(phase.status, `${filename}:${phase.id}: stale status`).toBe(
          statusFromTasks(tasks),
        );
      }

      const milestoneStatuses = [] as VersionStatus[];
      for (const milestone of document.milestones ?? []) {
        assertVersionStatus(milestone.status, `${filename}:${milestone.id}`);
        milestoneStatuses.push(milestone.status);
        await assertMilestoneSourceIntegrity(milestone, filename);
      }

      const childStatuses = document.milestones
        ? milestoneStatuses
        : phaseStatuses;
      expect(childStatuses, `${filename}: status has no children`).not
        .toHaveLength(0);
      expect(document.status, `${filename}: stale top-level status`).toBe(
        statusFromStatuses(childStatuses),
      );
    }
  });

  test("REQ-API-004: legacy feature registries are retired", async () => {
    // Mitase owns the feature graphs now. This guard keeps the retired
    // docs/spec tree from returning as a second authority.
    await expect(fs.stat(specRoot)).rejects.toThrow();
  });

  test("REQ-API-013: MCP documentation describes the shipped semantic facade", async () => {
    const source = await fs.readFile(
      path.join(repoRoot, "docs/architecture/api/mcp.md"),
      "utf8",
    );
    expect(source).toContain("POST /mcp");
    expect(source).toContain("ugoite.search");
    expect(source).toContain("ugoite://entry/{id}");
    expect(source).toContain("/.well-known/oauth-protected-resource");
    expect(source).toMatch(/DPoP/i);
  });
});

type VersionStatus = "planned" | "in_progress" | "completed";

type VersionTask = { done?: boolean };

type VersionPhase = {
  id: string;
  status: VersionStatus;
  tasks?: VersionTask[];
};

type VersionMilestone = {
  id: string;
  status: VersionStatus;
  source: string[];
  phases: Array<{ id: string; status: VersionStatus }>;
};

type VersionDocument = {
  status?: string;
  phases?: VersionPhase[];
  milestones?: VersionMilestone[];
};

const versionStatuses = new Set<VersionStatus>([
  "planned",
  "in_progress",
  "completed",
]);

function assertVersionStatus(
  status: string,
  owner: string,
): asserts status is VersionStatus {
  expect(
    versionStatuses.has(status as VersionStatus),
    `${owner}: invalid version status`,
  ).toBe(
    true,
  );
}

function statusFromTasks(tasks: VersionTask[]): VersionStatus {
  return statusFromStatuses(
    tasks.map((task) => (task.done === true ? "completed" : "planned")),
  );
}

function statusFromStatuses(statuses: VersionStatus[]): VersionStatus {
  if (statuses.every((status) => status === "completed")) return "completed";
  if (statuses.every((status) => status === "planned")) return "planned";
  return "in_progress";
}

async function assertMilestoneSourceIntegrity(
  milestone: VersionMilestone,
  owner: string,
): Promise<void> {
  expect(milestone.source, `${owner}:${milestone.id}: missing source`).not
    .toHaveLength(0);
  for (const source of milestone.source) {
    await expectPath(
      path.resolve(repoRoot, source),
      `${owner}:${milestone.id}`,
    );
  }

  const yamlSource = milestone.source.find((source) =>
    source.endsWith(".yaml")
  );
  expect(yamlSource, `${owner}:${milestone.id}: missing canonical YAML source`)
    .toBeDefined();
  const canonical = parse(
    await fs.readFile(path.resolve(repoRoot, yamlSource as string), "utf8"),
  ) as VersionDocument;
  expect(milestone.status, `${owner}:${milestone.id}: stale milestone status`)
    .toBe(
      canonical.status,
    );
  expect(
    milestone.phases.map(({ id, status }) => ({ id, status })),
    `${owner}:${milestone.id}: stale phase status summary`,
  ).toEqual(canonical.phases?.map(({ id, status }) => ({ id, status })));
}

async function yamlFiles(directory: string): Promise<string[]> {
  return (await fs.readdir(directory))
    .filter((filename) => filename.endsWith(".yaml"))
    .sort();
}

async function yamlFilesRecursively(
  directory: string,
  prefix = "",
): Promise<string[]> {
  const entries = await fs.readdir(directory, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const relativePath = path.join(prefix, entry.name);
    if (entry.isDirectory()) {
      files.push(
        ...await yamlFilesRecursively(
          path.join(directory, entry.name),
          relativePath,
        ),
      );
    } else if (entry.name.endsWith(".yaml")) {
      files.push(relativePath);
    }
  }
  return files.sort();
}

async function expectPath(filePath: string, owner: string): Promise<void> {
  await expect(fs.stat(filePath), `${owner}: missing ${filePath}`).resolves
    .toBeDefined();
}
