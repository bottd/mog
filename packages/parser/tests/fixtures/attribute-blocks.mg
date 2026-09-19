``attr:
title "My Document"
authors "John" "Jane"
``

``attr:
date "2026-04-15"
version 1
``

Root level attribute blocks merge into the document.

-
``attr:
color "red"
``
List item with attributes
-

-red:
``attr:
weight 2
``
Chained and block attributes combine
-

# Inline marker owns the block directly below it
``attr:
attached "to heading"
``

#
``attr:
class "note"
``
Block form marker parents the attribute block
#

``rust:attr:
attr is not the only attribute, so this stays verbatim
``

``attr:
``

-| a || b ||
``attr:
caption "between rows"
``
-| c || d ||

``attr:
node "unterminated
``

Attributes attach to the paragraph they interrupt
``attr:
color "red"
``

Attributes after a blank line belong to the document instead

``attr:
scope "document"
``

A paragraph resumed
``attr:
resumed "across the block"
``
after a block keeps the space the line break stands for.

A delimiter left open **across
``attr:
spans "the block"
``
the block** still closes, and the paragraph owns the attributes.
