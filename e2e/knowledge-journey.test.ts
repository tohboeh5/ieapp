/**
 * JOURNEY-KNOWLEDGE-001: Golden Knowledge Journey (Frontend evidence).
 *
 * Space create -> Form establish -> Entry create -> Entry edit -> Search ->
 * History -> Restore -> Reopen, proving the same durable Knowledge outcome
 * the CLI adapters must reach in C2/C3. Each checkpoint asserts the durable
 * postcondition through the canonical read surface, not the interaction
 * that produced it. Restore must append a new revision; history never
 * shortens.
 */

import { type APIRequestContext, expect, test } from "@playwright/test";
import { getBackendUrl, waitForServers } from "./lib/client.ts";

test.describe("JOURNEY-KNOWLEDGE-001", () => {
  const stamp = Date.now();
  const spaceSlug = `journey-space-${stamp}`;
  const formName = `JourneyForm-${stamp}`;
  const needle = `journey-needle-${stamp}`;
  const titleV1 = `Journey entry ${stamp}`;
  const titleV2 = `Journey entry edited ${stamp}`;
  const bodyV1 = "journey v1 body";
  const bodyV2 = "journey v2 body";

  let spaceId = "";
  let entryId = "";
  let rev1 = "";
  let rev2 = "";

  test.beforeAll(async ({ request }) => {
    await waitForServers(request);
  });

  async function resolveSpaceId(
    request: APIRequestContext,
    slug: string,
  ): Promise<string> {
    const listRes = await request.get(getBackendUrl("/spaces"));
    expect(listRes.ok()).toBe(true);
    const spaces = (await listRes.json()) as Array<{
      id: string;
      slug?: string;
    }>;
    const space = spaces.find((candidate) => candidate.slug === slug);
    expect(space).toBeDefined();
    return space!.id;
  }

  test("JOURNEY-KNOWLEDGE-001: Space create is durable and reopenable", async ({ request }) => {
    const createRes = await request.post(getBackendUrl("/spaces"), {
      data: { slug: spaceSlug, name: "Knowledge journey space" },
    });
    expect([200, 201, 409]).toContain(createRes.status());
    if (createRes.status() === 409) {
      spaceId = await resolveSpaceId(request, spaceSlug);
    } else {
      const created = (await createRes.json()) as { id?: string };
      expect(created.id).toBeTruthy();
      spaceId = created.id!;
    }
    expect(spaceId).not.toBe(spaceSlug);

    // Reopen: the same durable Space reads back identically.
    const getRes = await request.get(getBackendUrl(`/spaces/${spaceId}`));
    expect(getRes.ok()).toBe(true);
    const reopened = (await getRes.json()) as {
      id?: string;
      slug?: string;
    };
    expect(reopened.id).toBe(spaceId);
    expect(reopened.slug).toBe(spaceSlug);
  });

  test("JOURNEY-KNOWLEDGE-001: Form establish carries equivalent schema semantics", async ({ request }) => {
    const formDef = {
      name: formName,
      version: 1,
      template: `# ${formName}\n\n## Status\n\n## Body\n`,
      fields: {
        Status: { type: "string", required: true },
        Body: { type: "markdown", required: false },
      },
    };
    const createRes = await request.post(
      getBackendUrl(`/spaces/${spaceId}/forms`),
      { data: formDef },
    );
    expect([200, 201, 409]).toContain(createRes.status());

    await expect
      .poll(
        async () => {
          const listRes = await request.get(
            getBackendUrl(`/spaces/${spaceId}/forms`),
          );
          if (!listRes.ok()) return false;
          const forms = (await listRes.json()) as Array<{ name?: string }>;
          return forms.some((form) => form.name === formName);
        },
        { timeout: 30_000 },
      )
      .toBe(true);

    const getRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/forms/${encodeURIComponent(formName)}`),
    );
    expect(getRes.ok()).toBe(true);
    const form = (await getRes.json()) as {
      name?: string;
      fields?: Record<string, { type?: string; required?: boolean }>;
    };
    expect(form.name).toBe(formName);
    expect(form.fields?.Status?.type).toBe("string");
    expect(form.fields?.Status?.required).toBe(true);
    expect(form.fields?.Body?.type).toBe("markdown");
  });

  test("JOURNEY-KNOWLEDGE-001: Entry create appends exactly one revision", async ({ request }) => {
    const markdown =
      `---\nform: ${formName}\n---\n# ${titleV1}\n\n## Status\n${needle}\n\n## Body\n${bodyV1}\n`;
    const createRes = await request.post(
      getBackendUrl(`/spaces/${spaceId}/entries`),
      { data: { markdown } },
    );
    expect(createRes.status()).toBe(201);
    const created = (await createRes.json()) as {
      id?: string;
      revision_id?: string;
    };
    expect(created.id).toBeTruthy();
    expect(created.revision_id).toBeTruthy();
    entryId = created.id!;
    rev1 = created.revision_id!;

    const historyRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/history`),
    );
    expect(historyRes.ok()).toBe(true);
    const history = (await historyRes.json()) as {
      revisions?: Array<{ revision_id?: string }>;
    };
    expect(history.revisions).toHaveLength(1);
    expect(history.revisions?.[0]?.revision_id).toBe(rev1);
  });

  test("JOURNEY-KNOWLEDGE-001: Entry edit enforces optimistic concurrency", async ({ request }) => {
    const entryUrl = getBackendUrl(`/spaces/${spaceId}/entries/${entryId}`);
    const markdown =
      `---\nform: ${formName}\n---\n# ${titleV2}\n\n## Status\n${needle}\n\n## Body\n${bodyV2}\n`;
    const updateRes = await request.put(entryUrl, {
      data: { markdown, parent_revision_id: rev1 },
    });
    expect(updateRes.ok()).toBe(true);
    const updated = (await updateRes.json()) as { revision_id?: string };
    expect(updated.revision_id).toBeTruthy();
    expect(updated.revision_id).not.toBe(rev1);
    rev2 = updated.revision_id!;

    const staleRes = await request.put(entryUrl, {
      data: { markdown, parent_revision_id: rev1 },
    });
    expect(staleRes.status()).toBe(409);
  });

  test("JOURNEY-KNOWLEDGE-001: Search finds the updated durable entry", async ({ request }) => {
    const entryRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}`),
    );
    expect(entryRes.ok()).toBe(true);

    await expect
      .poll(
        async () => {
          const searchRes = await request.get(
            getBackendUrl(
              `/spaces/${spaceId}/search?q=${encodeURIComponent(needle)}`,
            ),
          );
          if (!searchRes.ok()) return false;
          const rows = (await searchRes.json()) as Array<{ id?: string }>;
          return rows.some((row) => row.id === entryId);
        },
        { timeout: 30_000 },
      )
      .toBe(true);
  });

  test("JOURNEY-KNOWLEDGE-001: History stays append-only across create and edit", async ({ request }) => {
    const historyRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/history`),
    );
    expect(historyRes.ok()).toBe(true);
    const history = (await historyRes.json()) as {
      revisions?: Array<{ revision_id?: string }>;
    };
    expect(history.revisions).toHaveLength(2);
    const ids = (history.revisions ?? []).map((item) => item.revision_id);
    expect(ids).toContain(rev1);
    expect(ids).toContain(rev2);
  });

  test("JOURNEY-KNOWLEDGE-001: Restore appends a new revision without shortening history", async ({ request }) => {
    const restoreRes = await request.post(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/restore`),
      { data: { revision_id: rev1 } },
    );
    expect(restoreRes.ok()).toBe(true);

    const historyRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/history`),
    );
    expect(historyRes.ok()).toBe(true);
    const history = (await historyRes.json()) as {
      revisions?: Array<{ revision_id?: string }>;
    };
    expect(history.revisions).toHaveLength(3);
    const ids = (history.revisions ?? []).map((item) => item.revision_id);
    expect(ids).toContain(rev1);
    expect(ids).toContain(rev2);
    const rev3 = ids.find((id) => id !== rev1 && id !== rev2);
    expect(rev3).toBeTruthy();

    // The appended revision replays rev1 content instead of rolling back.
    const revisionRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/history/${rev3}`),
    );
    expect(revisionRes.ok()).toBe(true);
    const revision = (await revisionRes.json()) as {
      revision_id?: string;
      markdown?: string;
    };
    expect(revision.revision_id).toBe(rev3);
    expect(revision.markdown).toContain(bodyV1);
    expect(revision.markdown).toContain(titleV1);
  });

  test("JOURNEY-KNOWLEDGE-001: Reopen reads identical durable state", async ({ request }) => {
    const spaceRes = await request.get(getBackendUrl(`/spaces/${spaceId}`));
    expect(spaceRes.ok()).toBe(true);
    const space = (await spaceRes.json()) as { id?: string };
    expect(space.id).toBe(spaceId);

    const historyRes = await request.get(
      getBackendUrl(`/spaces/${spaceId}/entries/${entryId}/history`),
    );
    expect(historyRes.ok()).toBe(true);
    const history = (await historyRes.json()) as {
      revisions?: Array<{ revision_id?: string }>;
    };
    expect(history.revisions).toHaveLength(3);

    const searchRes = await request.get(
      getBackendUrl(
        `/spaces/${spaceId}/search?q=${encodeURIComponent(needle)}`,
      ),
    );
    expect(searchRes.ok()).toBe(true);
    const rows = (await searchRes.json()) as Array<{ id?: string }>;
    expect(rows.some((row) => row.id === entryId)).toBe(true);
  });
});
