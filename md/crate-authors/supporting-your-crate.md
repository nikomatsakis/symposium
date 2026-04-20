# Supporting your crate

If you maintain a Rust crate, you can extend Symposium with skills, MCP servers, or other extensions that will teach agents the best way to use your crate.

## Support begins with skills

The easiest way to extend symposium is by adding **custom skills** for your crate. [If this is all you want to do, read the instructions for uploading skills here.](./publishing-skills.md).

## Symposium plugins give you more power

If you want more power, you can author a Symposium plugin. A plugin lets you configure skills, yes, but also hooks (callbacks at particular likes, like before or after tool use), MCP servers, and so forth. The instructions for authoring a plugin are found here.

## What you can provide

There are three kinds of extensions you can publish:

- **Standalone skills** — guidance documents that AI assistants receive automatically when a user's project depends on your crate.
- **Hooks** — checks and transformations that run when the AI performs certain actions, like writing code or running commands.
- **MCP servers** — tools and resources exposed to agents via the Model Context Protocol.

## Just want to add a skill?

If all you need is to publish guidance for your crate, you don't need to set up a plugin. Just write a `SKILL.md` with a few lines of frontmatter (name, description, which crate it's for) and a markdown body, then open a PR to the [symposium-dev/recommendations](https://github.com/symposium-dev/recommendations) repository.

See [Publishing skills](./publishing-skills.md) for the details.

## Need hooks, or hooks and skills?

If you want to publish hooks — or a combination of hooks and skills — you'll need to create a plugin. A plugin is a TOML manifest that ties everything together.

See [Creating a plugin](./creating-a-plugin.md) for how to set one up.

## Want to expose tools via MCP?

If your crate has an MCP server, you can register it through a plugin so agents discover it automatically.

See [Publishing MCP servers](./publishing-mcp-servers.md) for details.
