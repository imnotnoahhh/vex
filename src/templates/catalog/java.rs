use crate::templates::{MergeStrategy, TemplateFile};

use super::template_file;

pub(super) fn build() -> Vec<TemplateFile> {
    vec![
        template_file(
            ".tool-versions",
            "java 21\n",
            Some(MergeStrategy::ToolVersions),
        ),
        template_file(
            ".vex.toml",
            r#"[commands]
build = "mkdir -p out && javac -d out src/Main.java"
run = "mkdir -p out && javac -d out src/Main.java && java -cp out Main"
test = "mkdir -p out && javac -d out src/Main.java tests/MainSmoke.java && java -cp out MainSmoke"
"#,
            None,
        ),
        template_file(".gitignore", "out/\n", Some(MergeStrategy::GitIgnore)),
        template_file(
            "src/Main.java",
            r#"public class Main {
    public static void main(String[] args) {
        System.out.println("hello from vex");
    }
}
"#,
            None,
        ),
        template_file(
            "tests/MainSmoke.java",
            r#"public class MainSmoke {
    public static void main(String[] args) {
        System.out.println("add a Java test runner here");
    }
}
"#,
            None,
        ),
    ]
}
