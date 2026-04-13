"""update_readme.py — Updates the GitHub profile README with the generated SVG heatmap.

Implementation: Epic 5, Story 5.3.
"""

import argparse
import pathlib
import re
import sys

START_MARKER = "<!-- vibestats-start -->"
END_MARKER = "<!-- vibestats-end -->"

PATTERN = re.compile(
    r"(<!-- vibestats-start -->)(.*?)(<!-- vibestats-end -->)",
    re.DOTALL,
)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Inject vibestats heatmap into profile README")
    p.add_argument("--username", required=True, help="GitHub username")
    p.add_argument("--readme-path", default="README.md", help="Path to profile README")
    return p.parse_args()


def build_block(username: str) -> str:
    return (
        f"<!-- vibestats-start -->\n"
        f'<img src="https://raw.githubusercontent.com/{username}/{username}/main/vibestats/heatmap.svg"'
        f' alt="vibestats heatmap" />\n\n'
        f"[View interactive dashboard →](https://vibestats.dev/{username})\n"
        f"<!-- vibestats-end -->"
    )


def main() -> None:
    args = parse_args()
    if not args.username or not args.username.strip():
        print("ERROR: --username cannot be empty", file=sys.stderr)
        sys.exit(1)
    readme_path = pathlib.Path(args.readme_path)

    # Read the README file
    if not readme_path.exists():
        print(f"ERROR: README not found at {readme_path}", file=sys.stderr)
        sys.exit(1)

    try:
        content = readme_path.read_text(encoding="utf-8")
    except OSError as e:
        print(f"ERROR: could not read {readme_path}: {e}", file=sys.stderr)
        sys.exit(1)

    # Locate markers
    match = PATTERN.search(content)
    if match is None:
        print(
            "ERROR: vibestats markers not found in README. "
            "Add <!-- vibestats-start --> and <!-- vibestats-end --> to your profile README.",
            file=sys.stderr,
        )
        sys.exit(1)

    # Build the replacement block
    new_block = build_block(args.username)

    # Compare existing block to new block (strip to normalise whitespace)
    existing_block = match.group(0)
    if existing_block.strip() == new_block.strip():
        print("vibestats: README already up to date — skipping commit")
        sys.exit(0)

    # Replace content. Use a lambda replacement so `new_block` is treated as a
    # literal string — `re.sub` otherwise interprets backslash sequences (e.g.
    # `\1`, `\g<1>`) in the replacement, which could corrupt output if the
    # username ever contained such characters.
    updated_content = PATTERN.sub(lambda _m: new_block, content, count=1)

    # Write updated README
    try:
        readme_path.write_text(updated_content, encoding="utf-8")
    except OSError as e:
        print(f"ERROR: could not write {readme_path}: {e}", file=sys.stderr)
        sys.exit(1)

    print("vibestats: README updated")


if __name__ == "__main__":
    main()
