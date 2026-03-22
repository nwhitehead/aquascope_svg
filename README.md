# Kaya - memory state diagram tool

Kaya is a set of tools for producing beautiful memory state diagrams for small
example programs.

Kaya introduces a human readable diagram text format that can be used to create
diagrams for any language. The diagrams created can be in HTML, SVG, or PNG
format and aim to be high quality for publication online or in print. Kaya can
also connect to the Aquascope project to automatically analyze Rust code and
produce diagrams.

## Kaya Format

Kaya is a human-readable text format that uses some Markdown conventions and
encodes program state at various points.

Here is an example `kaya` diagram description:
```
# L1
## Stack
### main
x: ptr(H0)
y: ptr(H1)
## Heap
H0: 1
H1: (4, ptr(H2))
##
H2: (2, *)

# L2
## Stack
### main
x: ptr(H0)
y: *
z: ptr(H2.0).ds
## Heap
H0: 1
H1: (4, ptr(H2))
##
H2: (2, *)
```

This will render to:

![Kaya diagram showing two memory states with pointers](docs/demo.png)

See [FORMAT.md](docs/FORMAT.md) for details on the `kaya` diagram description format.

## Workflows

### Simple Workflow

The simplest workflow for `kaya` is:

```
Kaya format file --> HTML/SVG/PNG diagram
```

In this workflow you write a `kaya` diagram description directly in a text
editor then use the `render_kaya` tool to create the output HTML/SVG/PNG file.
The diagram can then be included inside other documentation.

If a build system is being used then changes to the `kaya` source files can
trigger rebuilds of the output diagrams.

### Kaya in Markdown Workflow

It is also possible to put `kaya` format diagrams inside other Markdown
documents. The workflow then becomes:

```
Markdown -> extrated Kaya --> HTML/SVG/PNG diagram
```

In this case the locations where `kaya` diagrams appear in the source text can
be code fences in the Markdown source tagged with `kaya`. This workflow
requires a Markdown processor plugin that extracts the `kaya` blocks,
calls `render_kaya` to get HTML/SVG/PNG output, then includes the output
in the final rendered Markdown.

### Automatic Rust Analysis Workflow

The general flow:

```
Rust code --> JSON analysis --> Kaya format --> HTML/SVG/PNG diagram
```

#### Details

The [Aquascope](https://github.com/cognitive-engineering-lab/aquascope) project
does the Rust code analysis (Rust to JSON). This can be a bit involved since
it involves compiling the snippet with a custom Rust compiler that allows some
types of errors and looking at the generated intermediate bytecode to extract
program state.

The Rust code analysis is saved as JSON data using the `aquascope_cli` tool in
[this fork](https://github.com/nwhitehead/aquascope). You give it a file
containing a short Rust program and it outputs the JSON analysis data to
`stdout`.

Next, this project has a tool `aquascope_json_to_kaya` for converting the JSON
format to `kaya` format. 

This project has a tool `render_kaya` for rendering `kaya` diagrams into
HTML/SVG/PNG. Usually this will be called from some sort of document preparation
system (e.g. during Markdown rendering).

#### Markdown plugin

An example workflow would be a workflow where you write content in Markdown
including code fences with Rust code examples. Some code blocks are marked as
`rust` and are included in the output just with syntax highlighting. Code blocks
where you want to include diagrams are also annotated with `aquascope`. The
Markdown renderer would extract the code block, pass it to `aquascope_cli`, pass
the JSON result to `aquascope_json_to_kay` to get the `kaya` diagram, then
render the `kaya` diagram using `render_kaya`. The plugin would then put the
Rust code in the output with syntax highlighting followed by the final diagram.

A Markdown plugin doing this is outside the scope of this project.

## Options

The `render_kaya` tool has the following options:

```
A tool for rendering Kaya diagrams

Usage: render_kaya [OPTIONS] --output <OUTPUT> <INPUT>

Arguments:
  <INPUT>  Input filename

Options:
      --show-parse       Parse the input and show debug parsing output to stdout
      --output-html      Output an HTML fragment
      --inline-js        Inline JS dependencies (default is to reference a CDN)
      --show-heap        Show labels starting with H (heap) (default is to hide)
      --output <OUTPUT>  Output filename, required (use - for stdout)
  -h, --help             Print help
```

## Building from source

In general `kaya` is a Rust project that uses standard Rust tools through
`cargo`.

To build:

```bash
cargo build --release
```

You can run some simple tests with:

```bash
cargo test
```

## Testdata

There are test Rust programs in `testdata/rust`.

JSON output from `aquascope_cli` is in `testdata/json`.
