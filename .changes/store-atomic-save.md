---
"store": patch
"store-js": patch
---

Save the store by writing a temporary file next to it, flushing it to disk and renaming it over the store file. Writing the file in place truncated it first, so a power loss or crash during a save could leave it empty or filled with zero bytes.
