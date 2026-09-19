An inline attribute block ``attr: color "red"`` attaches to its paragraph.

Inside **a delimiter ``attr: weight 2`` the delimiter** is the parent.

# A heading ``attr: class "note"``

#``attr: class "note"`` a delimiter flush against a marker is not a chain entry

Separate KDL nodes with a semicolon ``attr: a 1; b 2``

An inline verbatim with another attribute ``rust: let x = 1;`` stays verbatim.

-| a ``attr: align "end"`` || b ||

``attr: only "attributes"``

-| ``attr: align "end"`` || b ||

``(t)attr: typed stays verbatim``

**``attr: weight 2`` directly after the opener** the delimiter is still the parent.

**attr: on any other delimiter** is an ordinary attribute named attr.

**directly before the closer ``attr: weight 2``** the delimiter is still the parent.

A link written across a block [[target]]``attr: c "x"``((display)) still owns its name.

``attr: a 1`` ``attr: b 2`` text after two leading blocks has no leading space.

**spaced before the closer ``attr: weight 2`` ** leaves no empty text.

A spaced block between [[target]] ``attr: c 1`` ((display)) leaves one space.
