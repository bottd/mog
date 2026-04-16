--
title "Mog Syntax Specification"
authors "Drake Bott"
version 0.1
--

# Mog Syntax Specification v0.1

## Introduction

Mog is a markup syntax designed for rich, interconnected documents:
knowledge bases, wikis, and publishing on the web. It draws from
Markdown, Org, and Neorg, but diverges with two-character delimiters,
explicit nesting, and inline attributes. Together these create
a unified grammar for expression. This document specifies the structure
and rules for markup in a Mog document.

## Table of Contents

#| Section || Title                        ||
-| 1       || Document Structure           ||
-| 2       || Structural Markers           ||
-| 3       || Attributes                   ||
-| 4       || Semantic Delimiters          ||

## 1 Document Structure

### 1.1 Text Encoding

Documents are UTF-8 encoded files with a ``.mg`` file extension. A
leading byte-order mark (U+FEFF) is stripped by the parser and not
considered document content. Line endings can be LF (U+000A) or CRLF
(U+000D U+000A), parsing normalizes to LF.

### 1.2 Whitespace

The following Unicode characters are considered whitespace:

- Tab (0x09)
- Space (0x20)

Whitespace is not used to structure Mog. Leading and trailing whitespace
is trimmed when parsing a document. The following headings parse
identically:

``mog:
# My heading
  # My heading  
``

### 1.3 Paragraphs

A paragraph is a continuous sequence of text lines. Paragraphs are
terminated by a blank line, structural marker, or the end of the
document. Line breaks within a paragraph are parsed as soft wraps.

### 1.4 Metadata

Metadata may appear at the beginning of a document, delimited by two
hyphens. Data is formatted in [[https://kdl.dev]]((KDL)) syntax:

``mog:
--
title "My Document"
authors "John" "Jane"
date "2026-04-15"
version 1
--
``

### 1.5 Escape Sequences

A backslash immediately preceding a delimiter or structural marker
escapes it, rendering the delimiter or marker literal. A backslash in
any other position is itself literal and requires no escaping.

``mog:
\**this is not bold\**
``

## 2 Structural Markers

Structural markers are used to give documents structure. These markers
open the line they are used on, ignoring leading whitespace. Structural
markers are repeated to explicitly indicate nesting. Using a heading
``#`` marker once is a top-level heading, two markers ``##`` represent
a subheading and so on. These rules apply to all structural markers.

#| Marker   || AST Node        || Semantic Meaning             ||
-| ``#``    || heading         || Section heading              ||
-| ``-``    || unordered-list  || Unordered list item          ||
-| ``.``    || ordered-list    || Ordered list item            ||
-| ``>``    || blockquote      || Blockquote                   ||
-| ``=``    || thematic-break  || Thematic break               ||

An optional whitespace character can be placed between a marker
and content. This whitespace is trimmed when present.

### 2.1 Nesting

All structural markers can nest. Nesting is expressed through marker
repetition. A repeated marker indicates a sub-entity:

``mog:
# Heading 1
## Heading 2
### Heading 3
 ## Heading 2 with leading whitespace
  # Heading 1 with leading whitespace
``

### 2.2 Blocks

Structured text can span multiple lines. A structural marker alone on
a line with no content opens a block of its type, terminated with a
matching marker of the same depth on its own line.

``mog:
- Reading notes
--
Chapter 3 was particularly relevant:
 >
  quote from chapter 3
 >
--
``

Leading whitespace is trimmed while parsing structural markers. Authors
can use it to visually group text without affecting nesting depth. The
following block is equivalent to the above example:

``mog:
- Reading notes
--
Chapter 3 was particularly relevant:
>
quote from chapter 3
>
--
``

### 2.3 Tasks

A task marker can be placed after any structural marker except ``=``
thematic breaks. Tasks are marked using square brackets: ``[ ]``. The
status of a task is determined by the character between brackets.

#| Marker    || Status      ||
-| ``[ ]``   || Undone      ||
-| ``[x]``   || Done        ||
-| ``[~]``   || In progress ||
-| ``[?]``   || Uncertain   ||
-| ``[!]``   || Urgent      ||
-| ``[-]``   || Cancelled   ||

``mog:
- [~] Grocery shopping
-- [x] Eggs
-- [~] Milk
-- [?] Oat or almond?
- [ ] Clean kitchen
- [!] Call dentist
- [-] Return sweater
``

## 3 Attributes

Attributes are generic data attached to a marker or delimiter,
terminated using a colon. After the colon an optional whitespace
character is trimmed if present.

Attribute names may contain any character except whitespace and the
following reserved characters:

- ``:`` (0x3A)
- ``(`` (0x28)
- ``)`` (0x29)
- ``[`` (0x5B)
- ``]`` (0x5D)
- ``{`` (0x7B)
- ``}`` (0x7D)
- ``|`` (0x7C)
- ``;`` (0x3B)
- ``"`` (0x22)
- ``'`` (0x27)
- ``,`` (0x2C)
- ``~`` (0x7E)

All other printable characters, including digits and Unicode letters,
are permitted.

``mog:
##red:My Heading
##red: My Heading

- list item with **red: BOLD** content
``

### 3.1 Attribute Chains

Additional attributes are added as a chain beginning from the
terminating colon of the prior attribute. Attribute chains must stay on
the same line as their parent. A newline or whitespace character marks
the beginning of text content.

``mog:
##red:underline: My Heading
``

### 3.2 Data Attributes

Where an attribute flag is not enough, attributes can be structured
using [[https://janet-lang.org]]((Janet)) table or list syntax.

``mog:
-{:key "value"}: Table attribute
-["item 1" "item 2"]: List attribute
``

## 4 Semantic Delimiters

Semantic delimiters are used both inline and as blocks. All delimiters
open and close with two characters. Delimiters of different kinds can
nest, but delimiters cannot nest within themselves. Leading and trailing
whitespace is trimmed within semantic delimiters.

#| Delimiter      || AST Node        || Semantic Meaning    ||
-| ``**``         || strong          || Bold                ||
-| ``__``         || italic          || Italic              ||
-| `` `` ``       || verbatim        || Literal text        ||
-| ``~~``         || strikethrough   || Struck-through text ||
-| ``$$``         || math            || Math expression     ||
-| ``#|``         || table-header    || Table header        ||
-| ``-|``         || table-row       || Table row           ||
-| ``||``         || table-cell      || Table cell          ||
-| ``[[ ]]``      || link            || Link target         ||
-| ``(( ))``      || link-name       || Link name           ||
-| ``{{ }}``      || link-footnote   || Footnote content    ||

``mog:
The quick brown fox jumped over the **lazy dog.**

**
The quick brown fox jumped over the lazy dog.
**

A paragraph **with
soft wrapping** in
it.
``

### 4.1 Math

Text contained within math delimiters is treated as verbatim, but
evaluated as [[https://typst.app/docs/reference/math/]]((Typst)) math
when rendered.

``mog:
Einstein's famous equation $$E = m c^2$$ changed physics.

$$
E^2 = (m c^2)^2 + (p c)^2
$$
``

### 4.2 Verbatim

Text between verbatim delimiters is preserved literally. To indicate a
syntax for highlighting when rendered, a lang attribute is attached to
the marker or delimiter.

``mog:
Some code with \``python: print("Hello, world!")\``

\``python:
def greet(name):
    print(f"Hello, {name}!")
\``
``

Each line of a verbatim block is dedented by the whitespace preceding
the opening verbatim delimiter.

``mog:
-
  A list item
  \``python:
  def greet(name):
    print(f"Hello, {name}!")
  \``
-
``

When parsed, the verbatim text is dedented:

``python:
def greet(name):
  print(f"Hello, {name}!")
``

### 4.3 Tables

Tables are constructed using ``#|`` table header and ``-|`` table row
delimiters. Within a row, cells are delimited using ``||``. Table cells
can include semantic delimiters for presentation of tabulated data. A
table is terminated by an empty line.

``mog:
#| Name       || Type      || Color    ||
-| Apple      || Fruit     || Red      ||
-| **Carrot** || Vegetable || Orange   ||
-| Blueberry  || Fruit     || Blue     ||
``

Table rows may share a line.

``mog:
#| Name || Type || Color || -| Banana || Fruit || Yellow ||
``

Table rows are not required to have the same number of cells. The number
of columns in a table is determined by the row with the most cells.
Missing cells are parsed as empty. Implementations may warn on row
length mismatch.

``mog:
#| Name       || Type      || Color    ||
-| Apple      || Fruit     ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
-| Blueberry  || Fruit     || Blue     ||
``

Tables may begin without a header:

``mog:
-| Apple      || Fruit     ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
-| Blueberry  || Fruit     || Blue     ||
``

A table may have multiple or mid-table header rows:

``mog:
#|            ||           ||          || Texture ||
#| Name       || Type      || Color    ||
-| Apple      || Fruit     ||
-| Blueberry  || Fruit     || Blue     ||
#| Vegetables ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
``

### 4.4 Links

Links ``[[  ]]`` are formatted with square brackets. A link can be bare,
or it may have a name ``((  ))`` and footnote ``{{  }}``.

``mog:
- [[https://kdl.dev]]
- [[https://kdl.dev]]((KDL))
- [[https://kdl.dev]]((KDL)){{ A node-based document language }}
``

Link resolution is driven by attributes. A bare link is a document
reference and is resolved against filenames and metadata ``title``
fields. Headings can be referenced using the ``#`` attribute. External
targets are identified by their protocol attribute.

``mog:
[[recipe]]               Document reference
[[#:Ingredients]]        Heading in current document
[[https://kdl.dev]]      External via protocol attribute
``

Link transclusion is indicated using the transclude ``!`` attribute.
Transclusion works for both documents and media.

``mog:
[[!:recipe]]       Document Transclusion
[[!:image.png]]    Image Transclusion
``

### 4.5 Footnotes

When footnote contents do not fit inline, they may be linked from
elsewhere in the document:

``mog:
A good marinara starts with San Marzano
[[tomato]]((tomatoes)) and a generous amount
of olive oil.

[[tomato]]{{ A fruit not a vegetable }}
``

Footnotes can appear without a link. If unattached, a footnote is
indexed by its position in the document.

``mog:
A good marinara starts with San Marzano
tomato{{ A tomato is a fruit not a vegetable }} and a generous amount
of olive oil.
``
