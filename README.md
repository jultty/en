# en

en is a tool to write non-linear, connected pieces of text and have their references mapped out as a metadata-rich graph of connected information.

It works by ingesting a graph definition that leverages TOML for metadata and a special markup language for prose. en can then serve this textual representation as a website (with other targets planned) that allows nodes to be explored.

## Learn more

You can learn more and see what en looks like by visiting the [homepage](https://en.jutty.dev), which is rendered using en itself.

## Install and run

See the [Documentation](https://en.jutty.dev/node/Documentation) page for instructions and how to install and start using en.

## Roadmap

On a high-level, here are some things that en intends to achieve at some point:

1. Separate 'en' the server from 'en' the source-to-source translator. This would allow using en's markup syntax as a standalone compile-to-HTML language, effectively creating an "en cli" that should also have options to make it more practical to manipulate a graph, such as adding new nodes or querying your graph
3. Multifile graphs, with _optional_ TOML frontmatter
2. Make en more fitting to the 'note taking' use case
3. Multigraph, with the server being capable to let the user choose which graph to render and with inter-graph connections
4. Different rendering modes such as to a static website, to PDF or EPUB (with paper-friendly metadata)
5. Stateful and stateless online graph editing with 'export to TOML'

For a more detailed outline of what's planned for the future of en, along with what's already been completed, see the [roadmap](https://en.jutty.dev/node/Roadmap).
