``attr:
title "Mog Syntax Specification"
authors "Drake Bott"
version "0.2.0"
``

# Mog Syntax Specification

## Introduction

Mog is a markup language designed for rich, interconnected documents:
knowledge bases, wikis, and agentic workflows. Drawing from Markdown,
Org, and Neorg, the syntax uses two-character delimiters, explicit
nesting, and KDL attributes.

This document specifies the structure and rules for markup in a Mog
document. How a parsed document is resolved, rendered, or otherwise
processed is outside its scope.

## Table of Contents

#| 1   || Document Structure         ||
-| 1.1 || Text Encoding              ||
-| 1.2 || Whitespace and Blank Lines ||
-| 1.3 || Paragraphs                 ||
#| 2   || Structural Markers         ||
-| 2.1 || Depth                      ||
#| 3   || Semantic Delimiters        ||
-| 3.1 || Verbatim                   ||
-| 3.2 || Tables                     ||
-| 3.3 || Links                      ||
-| 3.4 || Annotations                ||
#| 4   || Boundaries                 ||
-| 4.1 || Blocks                     ||
-| 4.2 || Free                       ||
-| 4.3 || Trimming                   ||
-| 4.4 || Escapes                    ||
#| 5   || Attributes                 ||
-| 5.1 || Inline Attributes          ||
-| 5.2 || Verbatim Attributes        ||
-| 5.3 || Duplicate Attributes       ||

## 1 Document Structure

### 1.1 Text Encoding

A document is a UTF-8 encoded file with a ``.mg`` file extension.
Leading byte-order marks (U+FEFF) are skipped by the parser. Line
endings can be LF (U+000A) or CRLF (U+000D U+000A); parsing normalizes
to LF.

### 1.2 Whitespace and Blank Lines

The following Unicode characters are whitespace:

- Tab (U+0009)
- Space (U+0020)

A line is blank when it is empty or contains only whitespace.

### 1.3 Paragraphs

A paragraph is a continuous sequence of text lines. Paragraphs are
terminated by a blank line, structural marker (2), or the end of the
document. Line breaks within a paragraph are soft wraps: when parsed,
each line is joined by a single space.

## 2 Structural Markers

Structural markers open a line and give the following content a type.

#| Marker   || Node            || Semantic Meaning             ||
-| ``#``    || heading         || Section heading              ||
-| ``-``    || unordered-list  || Unordered list item          ||
-| ``.``    || ordered-list    || Ordered list item            ||
-| ``>``    || blockquote      || Blockquote                   ||
-| ``=``    || free            || Free                         ||

### 2.1 Depth

All structural markers have a depth expressed through marker
repetition. A repeated marker is one level deeper:

``mog:
# Heading 1
## Heading 2
### Heading 3
``

Depth is not used to nest nodes; containment is expressed through
block syntax (4.1).

## 3 Semantic Delimiters

Semantic delimiters mark or group text to give it meaning, style, or
structure. All delimiters open and close with two characters.
Delimiters of different kinds can nest, but a delimiter cannot nest
within itself.

#| Delimiter      || Node            || Semantic Meaning    ||
-| ``**``         || strong          || Bold                ||
-| ``__``         || italic          || Italic              ||
-| \``            || verbatim        || Literal text        ||
-| ``~~``         || strikethrough   || Struck-through text ||
-| ``::``         || free            || Free                ||
-| ``#|``         || table-header    || Table header        ||
-| ``-|``         || table-row       || Table row           ||
-| ``||``         || table-cell      || Table cell          ||
-| ``[[ ]]``      || hyperlink       || Hyperlink target    ||
-| ``<< >>``      || slot-link       || Slot link target    ||
-| ``(( ))``      || alias           || Alternate form      ||
-| ``{{ }}``      || note            || Additional note     ||

``mog:
The quick brown fox jumped over the **lazy dog.**

A paragraph __with
soft wrapping__ in
it.
``

### 3.1 Verbatim

Text between verbatim delimiters is preserved literally. A backslash
escape applies only to verbatim delimiters; all other text, including
backslashes, is preserved. When verbatim content ends with a backslash,
place whitespace before the closing delimiter; it is trimmed (4.3).

To specify a syntax, a language attribute (5.1) may be attached to the
opening delimiter:

``mog:
Some code with \``python: print("Hello, world!")\``

\``python:
def greet(name):
    print(f"Hello, {name}!")
\``
``

Each line of a verbatim block (4.1) is dedented by the whitespace
preceding the opening verbatim delimiter:

``mog:
-
  A list item
  \``python:
  def greet(name):
    print(f"Hello, {name}!")
  \``
-
``

When parsed, the remaining whitespace is preserved:

``python:
def greet(name):
  print(f"Hello, {name}!")
``

### 3.2 Tables

Tables are constructed using table header ``#|`` and table row ``-|``
delimiters. Within a row, cells are delimited using table cell ``||``.
Each cell may include semantic delimiters. A table is terminated by a
blank line (1.2).

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

Table rows are not required to have the same number of cells. The
number of columns in a table is determined by the row with the most
cells. Missing cells are parsed as empty.

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

Tables may have multiple header rows in any position:

``mog:
#|            ||           ||          || Texture ||
#| Name       || Type      || Color    ||
-| Apple      || Fruit     ||
-| Blueberry  || Fruit     || Blue     ||
#| Vegetables ||
-| Carrot     || Vegetable || Orange   || Crunchy ||
``

### 3.3 Links

A link is either a hyperlink ``[[ ]]`` or a slot link ``<< >>``. A
hyperlink is a reference the reader may follow to another resource. A
slot link is filled by its target, such as an image or template
variable. Links are inline only; no block form of either kind is valid.

``mog:
- [[https://kdl.dev]]
- [[spec.mg]]

- <<embedded-img.png>>
- <<my-note.mg>>
``

Attributes (5.1) on links conventionally declare the target's namespace:

``mog:
- [[#: 3.3 Links]]      heading namespace
- <<$:note.mg>>         project root namespace
- [[spell:Fireball]]    spell namespace
- <<ingredient:tomato>> ingredient namespace
``

### 3.4 Annotations

Annotations are alias ``(( ))`` and note ``{{ }}`` nodes. An alias
carries an alternate name or creates a named anchor point. A note is
for any attached context such as footnotes and comments.

Annotations are commonly attached to links:

``mog:
- [[https://kdl.dev]]((KDL doc site))
- [[https://kdl.dev]]{{
    KDL is a small, pleasant document language with XML-like node...
  }}
- [[https://kdl.dev]]((KDL doc site)){{a cuddly document language}}
``

Annotations can attach to a span or another annotation:

``mog:
- ::Groceries::{{ needed for [[marinara sauce]] }}
-- ::tomato::((San Marzano tomato))
-- olive oil
``

When an annotation appears inline with no preceding delimiter to attach
to, an anchor is registered at its position. For notes the anchor is
indexed by document order:

``mog:
A good marinara starts with San Marzano
tomato{{ A tomato is a fruit not a vegetable }} and a generous amount
of olive oil.
``

The prior example's note is indexed as ``1``. When the annotation is an
alias or note with attached alias, the anchor is keyed by the alias
name:

``mog:
A good marinara starts with {{ A tomato is a fruit not a vegetable }}((San Marzano tomato))
and a generous amount of olive oil.
``

An alias annotation may be used to name an anchor point for a following
note to target:

``mog:
A good marinara starts with ((San Marzano tomato)) and a generous
amount of olive oil.

{{ A fruit not a vegetable }}((San Marzano tomato))
``

## 4 Boundaries

### 4.1 Blocks

A marker or delimiter on an otherwise blank line opens a block. A block
is terminated by the same marker or delimiter with no attributes (5) on
an otherwise blank line.

``mog:
**
The quick brown fox jumped over the lazy dog.
**

# Reading notes
--
Chapter 3 was particularly relevant:
 >
 __
  quote from chapter 3
 __
 >
--
``

### 4.2 Free

A free node has no inherent semantic type. It exists as a structural
building block which carries attributes and annotations. The free
marker ``=`` denotes a line or block; the free delimiter ``::`` marks a
span.

``mog:
= A free line

=
 A free block

 can group content across lines
=

The sauce needs ::San Marzano::((a plum tomato)).
``

### 4.3 Trimming

Whitespace is not used to structure Mog syntax. Leading and trailing
whitespace is trimmed when parsing a document. The following headings
parse identically:

``mog:
# My heading
  # My heading  
      #     My heading  
#My heading          
``

Leading and trailing whitespace is trimmed within semantic delimiters,
except for verbatim content, which is trimmed only at its delimiters;
each contained line is dedented, not trimmed (3.1). The following bold
text spans contain the same parsed content:

``mog:
**    bold statement**
**bold statement    **
``

### 4.4 Escapes

A backslash escapes the character or run of repeated characters that
follows. Backslashes are the exception: ``\\ `` escapes a single
backslash, never a run. Within verbatim content, only a verbatim
delimiter is escaped (3.1).

``mog:
\**this is not bold\**
\\**this is bold**
\\\**this is not bold\**

\### this is not a heading
\---- this is not a list item
``

## 5 Attributes

Attributes attach generic KDL v2 data to nodes. An attribute can attach
inline or as a block. Inline attributes are terminated with a colon.

### 5.1 Inline Attributes

Inline attributes are KDL Nodes, separated and terminated by a colon
``:``. Each attribute may have a tag, name, and value. In a KDL
Document, ``key=value`` represents a node property. For Mog's
inline attributes an equals ``=`` instead names the node. This allows
inline and verbatim attributes (5.2) to interoperate.

``mog:
-size=medium: T-Shirt
-
 \``attr:
 size large
 \``
 Vintage T-Shirt with a block of text for its backstory
-
``

Both of these shirt notes carry a size attribute of the same format. If
the first T-Shirt was converted into a block using ``size medium`` to
store size, the representation of data is the same.

Additional attributes may be added as a chain beginning from the
terminating colon of the prior attribute. Attribute chains must stay on
the same line as their parent. A newline or whitespace character marks
the beginning of text content.

``mog:
-info: Item with "info" attribute
-info:green: Attribute chain
``

When a node has multiple attributes that share a key or tag, the values
can be comma ``,`` separated. Key and tag are repeated for each repeated
value:

``mog:
-(season)fall,winter:size=medium,small: Knit Beanie

## Equivalent Attributes
-(season)fall:(season)winter:size=medium:size=small: Knit Beanie
``

### 5.2 Verbatim Attributes

Verbatim attributes are declared by attaching an ``attr`` attribute to
an opening verbatim delimiter. A verbatim attribute is parsed as a KDL
Document, then attached to the containing node. At the top level, the
containing node is the document.

``mog:
\``attr:
title "My Document"
authors "John Doe"
\``

=
\``attr:
title "My Node"
authors "Also John Doe"
\``
=
``

### 5.3 Duplicate Attributes

When attributes share the same name they are not deduplicated. Instead,
all attributes are stored in document order.

``mog:
\``attr:
authors "John"
\``

-authors=James:authors=Jamie: Item

\``attr:
authors "Jane"
\``
``

This document has two ``authors`` attributes, ``John`` and ``Jane``.
The list node additionally has two separate ``authors`` attributes,
``James`` and ``Jamie``. Attributes with duplicate names are not merged
or overridden.
