"""agentic_capability_loop integration adapter for Context-Kernel.

This package provides a Python-level adapter for integration tests.
The production implementation uses the Rust crates in crates/ck-mcp/.

Context-Kernel emits:
  - ContextPacketCreated
  - ContextAccessEvaluated (context access, never action authority)

Context-Kernel consumes:
  - ContextRequired (from SWE_SEED)
"""
