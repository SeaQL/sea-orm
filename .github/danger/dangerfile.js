const { danger, fail, schedule } = require("danger");

const SELF_REVIEW_MARKER =
  "Remove this line to confirm you've reviewed this PR before submitting.";
const AI_GENERATED_RULE =
  "When writing repository content for library users or other developers, such as comments, documentation, or changelogs, prepend a standalone `AI-generated:` line in the file's native syntax.";
const AI_GENERATED_MARKER = AI_GENERATED_RULE.split("`")[1];

for (const file of [
  ...danger.git.modified_files,
  ...danger.git.created_files,
]) {
  schedule(async () => {
    const diff = await danger.git.diffForFile(file);
    if (!diff) {
      return;
    }
    if (file === "README.md" && diff.added.includes(SELF_REVIEW_MARKER)) {
      fail(
        "Please review your PR and manually remove the review marker from README.md before submitting.",
        file,
      );
    }
    if (
      diff.added.replaceAll(AI_GENERATED_RULE, "").includes(AI_GENERATED_MARKER)
    ) {
      fail(
        "Please review the AI-generated content and manually remove its notice before submitting.",
        file,
      );
    }
  });
}
