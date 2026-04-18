# Cavekit: Workspace

## Scope

Covers user identity, workspace creation, membership, and sharing. A workspace is the top-level container for all financial data. Users own workspaces and can invite others with defined roles. All ledger and baseline data is scoped to a workspace.

Membership state is modelled as an **append-only events log** (`membership_events`) with a **derived current-state projection** (`workspace_members`). The events log is never mutated — all membership changes are new rows. The current-state projection is maintained transactionally alongside each event write, and is the source of truth for permission checks. This provides a full audit trail and allows membership state to be reconstructed at any point in time.

## Requirements

### R1: User Accounts
**Description:** A user is an authenticated identity with a unique email address. Users can exist independently of any workspace.
**Acceptance Criteria:**
- [ ] A user can be created with a unique email and password
- [ ] Creating two users with the same email fails with a duplicate error
- [ ] A user has a unique identifier, email, display name, and timestamps (created, updated)
- [ ] User passwords are not stored in plaintext
**Dependencies:** None

### R2: Workspace Creation
**Description:** A user can create a workspace. The creator is automatically the owner. A user may belong to multiple workspaces.
**Acceptance Criteria:**
- [ ] Creating a workspace produces a workspace with a unique identifier, name, and timestamps
- [ ] The creating user is assigned the `owner` role in that workspace
- [ ] A user can create more than one workspace
- [ ] A workspace has at least one member (its owner) at all times
**Dependencies:** R1

### R3: Workspace Membership
**Description:** Workspace members have a role that determines what they can do. Roles are: `owner`, `editor`, `viewer`. Current membership state is derived from the membership events log — permission checks always read from the current-state projection.
**Acceptance Criteria:**
- [ ] An `owner` can invite members, remove members, and delete the workspace
- [ ] An `editor` can create, update, and delete ledger entries and baselines
- [ ] A `viewer` can read all workspace data but cannot modify anything
- [ ] A member cannot escalate their own role
- [ ] Removing the last owner from a workspace is prevented
- [ ] Every membership change (invite, accept, role change, removal, leave) is recorded as an immutable event in the membership events log
- [ ] The current-state projection always reflects the latest event per `(workspace, user)` pair
- [ ] Permission checks read from the current-state projection, not the events log directly
**Dependencies:** R2

### R4: Invitations
**Description:** An owner can invite a user to a workspace by email. The invited user must accept before gaining access.
**Acceptance Criteria:**
- [ ] An owner can send an invitation to an email address with a specified role
- [ ] An invitation has an expiry (minimum 48 hours)
- [ ] Accepting a valid invitation grants the invited role
- [ ] Accepting an expired invitation fails with an appropriate error
- [ ] A user already in the workspace cannot be re-invited
**Dependencies:** R3

### R5: Workspace Isolation
**Description:** Data in one workspace is never visible to members of another workspace, unless explicitly shared.
**Acceptance Criteria:**
- [ ] A member of workspace A cannot read any data from workspace B
- [ ] All queries for ledger and baseline data are scoped by workspace identifier
- [ ] An authenticated user without workspace membership receives a not-found or forbidden response for that workspace's data
**Dependencies:** R2

### R6: Leaving and Removing Members
**Description:** A member can leave a workspace. An owner can remove any non-owner member. The at-least-one-owner invariant must hold atomically — the check and the event write occur in a single transaction.
**Acceptance Criteria:**
- [ ] A non-owner member can leave a workspace voluntarily
- [ ] An owner can remove an editor or viewer
- [ ] An owner cannot remove themselves if they are the last owner
- [ ] An owner cannot be removed by another owner if they are the last owner
- [ ] The last-owner check and the membership event write occur within a single transaction — there is no window where a workspace can have zero owners
- [ ] Leaving a workspace does not delete any data the user contributed
- [ ] A leave or removal event is appended to the membership events log and the current-state projection is updated in the same transaction
**Dependencies:** R3

## Out of Scope
- OAuth / social login (future)
- Password reset flows (future)
- Workspace-level billing or subscription management (future)
- Audit logs (future)
- Fine-grained per-category or per-account permissions

## Cross-References
- Depended on by: cavekit-ledger.md — all accounts and transactions are scoped to a workspace
- Depended on by: cavekit-baselines.md — all baselines are scoped to a workspace
- Depended on by: cavekit-api.md — all endpoints require workspace membership
