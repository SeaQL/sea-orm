const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const dangerfile = fs.readFileSync(
  path.join(__dirname, "dangerfile.js"),
  "utf8",
);
const marker =
  "> Remove this line to confirm you've reviewed this PR before submitting.";
const aiMarker = "AI-generated" + ":";
const aiGeneratedRule =
  "When writing repository content for library users or other developers, such as comments, documentation, or changelogs, prepend a standalone `AI-generated:` line in the file's native syntax.";

for (const { name, modified, created, diffs, rejected = [] } of [
  {
    name: "checks the review marker only in the root README",
    modified: ["README.md", "docs/README.md"],
    diffs: {
      "README.md": { added: marker },
      "docs/README.md": { added: marker },
    },
    rejected: ["README.md"],
  },
  {
    name: "only checks newly added lines",
    modified: ["README.md", "src/expr.rs"],
    diffs: {
      "README.md": {
        added: "Updated documentation",
        removed: marker,
        diff: `-${marker}`,
      },
      "src/expr.rs": {
        added: "/// Updated documentation",
        removed: `/// ${aiMarker}`,
        diff: ` /// ${aiMarker}\n-/// ${aiMarker}`,
      },
    },
  },
  {
    name: "rejects AI markers in modified and newly created files",
    modified: [".github/danger/dangerfile.test.js"],
    created: ["CHANGELOG.md"],
    diffs: {
      ".github/danger/dangerfile.test.js": {
        added: `const marker = "${aiMarker}";`,
      },
      "CHANGELOG.md": { added: aiMarker },
    },
    rejected: [".github/danger/dangerfile.test.js", "CHANGELOG.md"],
  },
  {
    name: "allows the exact marker instruction",
    modified: ["AGENTS.md"],
    diffs: {
      "AGENTS.md": { added: aiGeneratedRule },
    },
  },
  {
    name: "rejects actual notices in files that also define the marker",
    modified: [".github/danger/dangerfile.js"],
    diffs: {
      ".github/danger/dangerfile.js": {
        added: `const rule = "${aiGeneratedRule}"; // ${aiMarker}`,
      },
    },
    rejected: [".github/danger/dangerfile.js"],
  },
]) {
  test(name, async () => {
    const scheduled = [];
    const failures = [];
    vm.runInNewContext(dangerfile, {
      require() {
        return {
          danger: {
            git: {
              modified_files: modified || [],
              created_files: created || [],
              async diffForFile(file) {
                return diffs[file];
              },
            },
          },
          fail: (_message, file) => failures.push(file),
          schedule: (task) => scheduled.push(task),
        };
      },
    });
    await Promise.all(scheduled.map((task) => task()));

    assert.deepEqual(failures.sort(), [...rejected].sort());
  });
}
