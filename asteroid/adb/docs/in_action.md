# This document will be completed when explicit mode becomes more mature.

## ADB in action
This document covers some of the finer details of ADB. In particular, the explicit
mode feature and how it allows for pattern debugging.

### Explicit Mode
Explicit mode is a feature of ADB that allows the user to understand and inspect
Asteroid's pattern matching. Pattern matching in Asteroid is generally silent. You
only really see the details of a pattern match when an error occurs. Explicit mode,
however, allows you to see every mattern matching operation that Asteroid executes.

Explicit mode aims to be readable. That being said, pattern matching, especially
list-based pattern matching, is complicated. Sometimes the messages from explicit
mode can be dense and difficult to read. Some reading tips are included at the
end of this document.