<div align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Stuart/master/example/static/lightning.png" width=150>

  <h3 align="center">Stuart</h3>

  <p align="center">
    A Blazingly-Fast Static Site Generator.<br>
    <a href="https://github.com/w-henderson/Stuart/releases"><strong>Download Now »</strong></a>
  </p><br>
</div>

<hr><br>

Stuart is a very fast and flexible static site generator, with build times as low as 0.1ms per page. It is written in Rust, and is designed be easier to use than other SSGs, while still beating them in the benchmarks. Stuart's simple yet powerful templating system allows you to define complex logic for your site, sourcing data from Markdown and JSON files as well as the template files, before rendering it all to static HTML. For even more complex projects, you can augment Stuart with custom build scripts in any language that integrate with the core build system. Support for plugins written in Rust is coming soon.

**Note:** Stuart is an extremely new project, so functionality and documentation are still being added. For the time being, documentation is limited to this README.

## Table of Contents

- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Creating a Project](#creating-a-project)
  - [Building a Project](#building-a-project)
  - [Configuration](#configuration)
- [Project Structure](#project-structure)
  - [The Root Template](#the-root-template)
  - [HTML and Markdown Pages](#html-and-markdown-pages)
  - [JSON Data](#json-data)
  - [Static Files](#static-files)
  - [Build Scripts](#build-scripts)
- [Templating Language](#templating-language)
  - [Variables](#variables)
  - [Functions](#functions)

## Getting Started

### Installation

Stuart is available as a pre-built binary for Windows and Linux. You can download the latest release from the [releases page](https://github.com/w-henderson/Stuart/releases). Alternatively, you can build the code from scratch using Rust's package manager, Cargo. To do this, clone the repository and run `cargo build --release`.

### Creating a Project

You can create a project by running `stuart new <project-name>`. This will create a new directory with the given name, and populate it with a basic project template. By default, Stuart will also initalise a Git repository in the project directory, so to avoid this behaviour, you can use the `--no-git` flag.

### Building a Project

You can build the project by running `stuart build` in the project directory. This will build the project into the `dist` directory.

To start the development server, which will automatically rebuild the project when files are changed and reload it in your browser, run `stuart dev`. This will start the server at [http://localhost:6904](http://localhost:6904).

### Configuration

In the `stuart.toml` file, you can set configuration options for your project in the `[settings]` section. The following options are available:

| Name | Description | Default |
| --- | --- | --- |
| `strip_extensions` | Whether to remove HTML file extensions by creating folders with `index.html` files | `true` |
| `save_data_files` | Whether to save the JSON data files to the output directory | `false` |
| `save_metadata` | Whether to output metadata about the build, used to integrate with build scripts | `false` |

## Project Structure

A Stuart project contains a number of folders, each of which has a specific purpose. Additionally, some file names have special meanings too. All content should go in the `content` directory, as this is the only one that will be processed by the build system.

### The Root Template

The root template is an HTML template file, called `root.html`, which serves as the basis for all other pages in its directory and in subdirectories. The most specific root template to the page being rendered will be used.

It is a regular HTML file which can contain template tags (which we'll discuss later), but must also contain the very important `insert` function at least once. This function takes a single argument, which is the name of a section. Sections are how Stuart knows where to insert content into the root template. This is most easily explained with an example.

`root.html`:

```html
<html>
  <head>
    {{ insert("head") }}
  </head>

  <body>
    {{ insert("body") }}
  </body>
</html>
```

In this contrived example, every page of the site will be rendered into this template. The `head` section of every page (marked by `begin` and `end` functions, which we'll see later) will be inserted into the head of the page, and the `body` into the body. This allows you to define a common layout for the site, or for a part of the site, and insert content into it.

It is important to note that sections are not optional: every section defined in the root template must appear in every page rendered with it.

A page that could be rendered into this template is as follows:

`index.html`:

```html
{{ begin("head") }}
<title>Stuart</title>
{{ end("head") }}

{{ begin("body") }}
<h1>Stuart</h1>
<p>A blazingly-fast static site generator.</p>
{{ end("body") }}
```

### HTML and Markdown Pages

HTML pages are regular HTML files, which can contain template tags. They define sections which are rendered into the root template.

Markdown pages are markdown files, starting with frontmatter containing metadata about the file, and then the content of the page. These are converted into HTML, rendered into the nearest `md.html` template through the `$self` variable, and the section from this are then rendered into the root template. Again, this is clearer with an example.

`my_page.md`:

```markdown
---
title: "My Page"
author: "William Henderson"
---

# My Page
Markdown content...
```

`md.html`:

```html
{{ begin("head") }}
<title>{{ $self.title }}</title>
{{ end("head") }}

{{ begin("body") }}
<h1>{{ $self.title }}</h1>
<p>By {{ $self.author }}</p>

{{ $self.content }}
{{ end("body") }}
```

`root.html` as above.

### JSON Data

JSON data files can also be sources of data for a Stuart website using the `import` templating function in an HTML page, which imports a JSON file as a variable.

### Static Files

Static files should be placed in the `static` directory, which is merged with the built content at the end of the build. Filename conflicts will cause the build to fail.

### Build Scripts

Build scripts should be placed in the `scripts` directory. Currently, the only scripts that Stuart supports are `onPreBuild` and `onPostBuild`. On Windows, these should have `.bat` extensions, and on Linux, they should have either `.sh` extensions or no extension at all. These scripts are run before and after the build, respectively.

The `onPostBuild` script can access metadata about the build in the `metadata.json` file, if `save_metadata` is enabled in the project configuration.

If a script wants to create files in the output directory, it should do so in the `temp` directory, which Stuart will merge into the output directory at the end of the build. This is to avoid conflicts with the build system, as writing directly to the output directory could cause unexpected behaviour.

## Templating Language

The Stuart templating language consists of two main parts: variables and functions. Variables are used to insert data into the template, and functions allow for more complex behaviour such as iteration and selection.

All template tags are enclosed in double curly braces. Variables are prefixed with a dollar sign.

### Variables

A basic variable can be inserted into the template as follows:

```html
{{ $variable }}
```

All variables are JSON values, so object values can be accessed using dot notation:

```html
{{ $variable.property }}
```

### Functions

Functions are called using the following syntax:

```html
{{ function_name(arg1, arg2, positional_arg="value", ...) }}
```

Stuart currently supports the following functions:

| Name | Description | Example(s) |
| --- | --- | --- |
| `begin` | Begins a section. | `begin("section_name")` |
| `end` | Ends a section or another function. | `end("section_name")`, `end(function_name)` |
| `insert` | Inserts a section into the template, only used in `root.html`. | `insert("section_name")` |
| `import` | Imports a JSON file as a variable. | `import($data, "data.json")` |
| `for` | Iterates over a JSON array or a directory of markdown files. The loop is ended with `end(for)`. | `for($tag, "tags.json")`, `for($post, "posts/", limit=3, order="desc", sortby="date")`, `for($item, $array)` |
| `dateformat` | Formats a date using the [chrono](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html) format string. The date input can be any kind of formatted date or timestamp. | `dateformat($date, "%Y-%m-%d")` |
| `ifdefined` | Checks if a variable is defined. The block is ended with `end(ifdefined)`. | `ifdefined($variable)`, `ifdefined($variable.property)` |
| `excerpt` | Creates an excerpt from a string. | `excerpt($post.content, 100)` |
| `timetoread` | Calculates the time to read a string in minutes. | `timetoread($post.content)` |