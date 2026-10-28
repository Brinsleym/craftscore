# Automation security model

- Loopback only; a 256-bit token per launch before any method runs.
- Separate read and write workspace roots; commands with paths outside them are refused.
- No method changes account credentials, sync backends or network permissions.
- Request-line, batch-step, connection and duration limits.
- Every automation-originated change is a normal command with a normal undo step.
