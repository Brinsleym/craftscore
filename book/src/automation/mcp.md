# MCP

`scorecraft-cli mcp` exposes every enabled command as an MCP tool, with its params doc as the input
description, plus read-only resources for the library, the open score and UI state. With
`--bridge` it forwards to the running app's control channel. Filesystem-bearing commands are
confined to the configured workspace roots.
