use crate::templates::{MergeStrategy, TemplateFile};

use super::template_file;

pub(super) fn build(project_name: &str) -> Vec<TemplateFile> {
    vec![
        template_file(
            ".tool-versions",
            "node 20\n",
            Some(MergeStrategy::ToolVersions),
        ),
        template_file(
            ".vex.toml",
            r#"[commands]
install = "npm install"
build = "npm run build"
test = "npm run test"
run = "npm run start"
"#,
            None,
        ),
        template_file(
            ".gitignore",
            "node_modules/\ndist/\n",
            Some(MergeStrategy::GitIgnore),
        ),
        template_file(
            "package.json",
            format!(
                r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "build": "tsc -p tsconfig.json",
    "test": "npm run build && node --test dist/tests/*.test.js",
    "start": "npm run build && node dist/src/index.js"
  }},
  "devDependencies": {{
    "@types/node": "^24.0.0",
    "typescript": "^5.8.0"
  }}
}}
"#,
                project_name
            ),
            None,
        ),
        template_file(
            "tsconfig.json",
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "rootDir": ".",
    "outDir": "dist",
    "strict": true,
    "esModuleInterop": true
  },
  "include": [
    "src/**/*.ts",
    "tests/**/*.ts"
  ]
}
"#,
            None,
        ),
        template_file(
            "src/index.ts",
            r#"export function greet(name: string): string {
  return `Hello, ${name}!`;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(greet("vex"));
}
"#,
            None,
        ),
        template_file(
            "tests/index.test.ts",
            r#"import test from "node:test";
import assert from "node:assert/strict";
import { greet } from "../src/index.js";

test("greet returns the expected value", () => {
  assert.equal(greet("vex"), "Hello, vex!");
});
"#,
            None,
        ),
    ]
}
