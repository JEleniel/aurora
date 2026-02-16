---
applyTo: '**/*'
---

# IDE Specific Instructions

- You are working within the Visual Studio Code IDE.
- This project may not be on GitHub; you MUST NOT assume it is.
- The specified tools (for example `markdownlint-cli2` and `prettier`) are installed globally. If you cannot run them, notify the user.
- Only when the user requests GitHub interactions (or the repo is confirmed to be on GitHub and the task requires it), use MCP tools. Do not use `gh`.

## Prohibited Actions

You MUST NOT, at any time, for any reason:

- Write to any folder outside the workspace. If you need temporary space, create `tmp/` within the workspace.
- Use `|| true`, `true ||`, or `true` as a command or part of a command, especially in the terminal.
- Use the `gh` command line tool. It is not installed and will not be.
- Use the `head` or `tail` commands in the terminal.
- Write or run ad-hoc scripts in any language, or invoke language runtimes (for example `python`, `node`, etc) for ad-hoc execution.
- Use chaining operators (for example `&&`, `;`). Pipes (`|`) are allowed.
