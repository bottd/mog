``attr:
title "first"
author "A"
``

``attr:
title "second"
tag "x"
tag "y"
authors {
  author "A"
  author "B"
}
``

Root blocks append into one children document and repeated names are kept, as in KDL.

-color="fromchain":
``attr:
color "fromblock"
weight 2
``
Chain entries and block children are separate namespaces, so both colors remain
-

##a=1:a=2: A repeated property in a chain resolves to the rightmost value

-red:underline: Arguments have no name to collide on
