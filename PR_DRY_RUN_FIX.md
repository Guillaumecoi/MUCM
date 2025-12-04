# Fix: Prevent dry-run from mutating use case state

## Problem
The `reinitialize` command's `--dry-run` flag was inadvertently modifying use case data in memory, causing the interactive mode to malfunction:

1. User runs dry-run → shows missing fields **but mutates in-memory state**
2. User confirms to apply changes
3. Actual run finds **0 updates** (fields already added in step 1)

This made the interactive reinitialize workflow completely non-functional.

## Root Cause
The code was calling `.entry().or_default()` on `HashMap` which creates empty entries even in read-only operations. This happened during dry-run when just checking for missing fields.

## Solution
- Separate field detection from field insertion into two passes
- Use `.get()` instead of `.entry().or_default()` for read-only checks  
- Only call `.entry().or_default().insert()` when `!dry_run`
- Added early exit pattern to reduce nesting and improve clarity

## Testing
- **Regression test added**: `test_dry_run_does_not_mutate_state` explicitly verifies the bug cannot resurface
- All 16 integration tests pass (field_reinitialization, reinitialize_command, check_command)
- All 553 tests pass in Docker CI environment (`act -j test`)
- Manual testing confirms interactive mode now works correctly

The regression test verifies:
- Dry-run does not change field count in memory
- No unexpected methodology entries are created
- Same use case can be checked twice without state corruption

## Changed Files
- `src/core/application/services/methodology_field_reinitialize_service.rs` (39 insertions, 18 deletions)

## Type
- [x] Bug fix (non-breaking change which fixes an issue)
