import { validateKnowledgeCompatibilityReview } from "./knowledge_compatibility.ts";

export const requiredSections = [
  "## Summary",
  "## Related Issue (required)",
  "## Knowledge Compatibility Review",
  "## Testing",
];

export const dependabotAuthor = "dependabot[bot]";

export function isExemptAuthor(author: string | undefined): boolean {
  return author === dependabotAuthor;
}

export function sectionText(body: string, heading: string): string {
  const match = body.match(
    new RegExp(`##\\s*${heading}\\s*\\n+([\\s\\S]*?)(?:\\n##\\s|$)`, "i"),
  );
  return match?.[1]?.trim() ?? "";
}

export function validatePrBody(body: string): string[] {
  const errors: string[] = [];

  for (const section of requiredSections) {
    if (!body.toLowerCase().includes(section.toLowerCase())) {
      errors.push(`Missing required section: ${section}`);
    }
  }

  const summaryText = sectionText(body, "Summary");
  if (
    !summaryText || summaryText === "-" || /^-\s*$/.test(summaryText)
  ) {
    errors.push(
      "Summary section must be filled in and cannot remain the '-' placeholder.",
    );
  }

  const issueLinkPattern =
    /(?:^|\n)\s*(?:close:\s*#\d+|closes\s+#\d+)\s*(?:\n|$)/i;
  if (!issueLinkPattern.test(body)) {
    errors.push(
      "Related Issue must include either `close: #123` or `closes #123`.",
    );
  }

  errors.push(...validateKnowledgeCompatibilityReview(body));

  const testingText = sectionText(body, "Testing");
  if (!/- \[(?: |x|X)\]/.test(testingText)) {
    errors.push("Testing section must include at least one checklist item.");
  }

  return errors;
}

export type PullRequestPointer = {
  body: string;
  author?: string;
};

export function extractPrNumberFromMergeGroupRef(ref: string): number | null {
  const match = ref.match(/(?:^|\/)pr-(\d+)(?:-|$)/);
  return match ? Number.parseInt(match[1], 10) : null;
}

export async function fetchPullRequestPointer(
  repo: string,
  prNumber: number,
  token: string,
): Promise<PullRequestPointer> {
  const response = await fetch(
    `https://api.github.com/repos/${repo}/pulls/${prNumber}`,
    {
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github+json",
      },
    },
  );
  if (!response.ok) {
    throw new Error(
      `Failed to load pull request #${prNumber}: HTTP ${response.status}`,
    );
  }
  const data = await response.json() as {
    body?: string | null;
    user?: { login?: string };
  };
  return { body: data.body ?? "", author: data.user?.login };
}

export async function resolvePullRequestPointer(env: {
  GITHUB_EVENT_NAME?: string;
  GITHUB_EVENT_PATH?: string;
  GITHUB_REF?: string;
  GITHUB_REPOSITORY?: string;
  GITHUB_TOKEN?: string;
}): Promise<PullRequestPointer> {
  if (env.GITHUB_EVENT_NAME === "merge_group") {
    const prNumber = extractPrNumberFromMergeGroupRef(env.GITHUB_REF ?? "");
    if (prNumber !== null && env.GITHUB_REPOSITORY && env.GITHUB_TOKEN) {
      return await fetchPullRequestPointer(
        env.GITHUB_REPOSITORY,
        prNumber,
        env.GITHUB_TOKEN,
      );
    }
    return { body: "" };
  }
  if (!env.GITHUB_EVENT_PATH) {
    return { body: "" };
  }
  const payload = JSON.parse(
    await Deno.readTextFile(env.GITHUB_EVENT_PATH),
  ) as { pull_request?: { body?: string | null; user?: { login?: string } } };
  return {
    body: payload.pull_request?.body ?? "",
    author: payload.pull_request?.user?.login,
  };
}

if (import.meta.main) {
  const pointer = await resolvePullRequestPointer(Deno.env.toObject());
  if (isExemptAuthor(pointer.author)) {
    console.log(
      "Dependabot pull requests use generated bodies; skipping the human PR template gate.",
    );
    Deno.exit(0);
  }
  const errors = validatePrBody(pointer.body);
  if (errors.length > 0) {
    console.error(errors.join("\n"));
    Deno.exit(1);
  }
  console.log("PR body matches the required template.");
}
