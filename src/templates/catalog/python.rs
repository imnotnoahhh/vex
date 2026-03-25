use crate::templates::{MergeStrategy, TemplateFile};

use super::template_file;

pub(super) fn build() -> Vec<TemplateFile> {
    vec![
        template_file(
            ".tool-versions",
            "python 3.12\n",
            Some(MergeStrategy::ToolVersions),
        ),
        template_file(
            ".vex.toml",
            r#"[commands]
init = "vex python init"
freeze = "vex python freeze"
sync = "vex python sync"
run = "python src/main.py"
test = "python -m unittest discover -s tests"
"#,
            None,
        ),
        template_file(
            ".gitignore",
            ".venv/\n__pycache__/\n*.pyc\n",
            Some(MergeStrategy::GitIgnore),
        ),
        template_file("requirements.lock", "", None),
        template_file(
            "src/main.py",
            r#"def greet(name: str) -> str:
    return f"Hello, {name}!"


if __name__ == "__main__":
    print(greet("vex"))
"#,
            None,
        ),
        template_file(
            "tests/test_main.py",
            r#"import unittest

from src.main import greet


class GreetTests(unittest.TestCase):
    def test_greet(self) -> None:
        self.assertEqual(greet("vex"), "Hello, vex!")


if __name__ == "__main__":
    unittest.main()
"#,
            None,
        ),
    ]
}
