# pgerd

Connects to a live Postgres database and renders its real foreign-key
relationships as a Graphviz DOT entity-relationship diagram. Complements
this workspace's `pgdiff` (which diffs two schemas) and `pgexplain`
(which visualizes a query plan) — this visualizes the *shape* of a
schema itself: which tables reference which, through which columns,
without hand-drawing an ERD or exporting one from a GUI tool.

## Installation

```sh
cargo install --path .
```

## Usage

```bash
pgerd --database-url postgres://user:pass@host/db > schema.dot
dot -Tpng schema.dot -o schema.png   # if Graphviz is installed

DATABASE_URL=postgres://user:pass@host/db pgerd --schema public
```

`--database-url` also reads from the `DATABASE_URL` environment
variable. Every table in the target schema (default `public`) becomes a
node — including one with no foreign keys at all, so an isolated table
still shows up rather than silently vanishing — and every real `FOREIGN
KEY` constraint becomes a labeled edge showing the exact column mapping.

## How it queries Postgres

Reads `pg_constraint` directly (not `information_schema`, which has
known rough edges around composite-key column ordering) filtered to
`contype = 'f'`, resolving each constraint's `conkey`/`confkey` column
arrays back to real column names via `pg_attribute`, ordinality-preserved
so a composite foreign key's column pairs stay correctly matched up
rather than cross-joined. `conrelid`/`confrelid` are cast through
`::regclass::text` for the real table names, which correctly handles a
non-`public` schema without extra join logic.

## Status: built and verified against a real live Postgres schema, not synthetic fixtures

- **6 unit tests** (`cargo test --lib`): a plain two-table relation
  renders as one labeled edge; a table with zero foreign keys still
  appears as an isolated node; a self-referencing foreign key renders as
  a genuine self-loop edge; a composite-key relation joins its column
  pairs with commas in the label; the output is a well-formed `digraph`
  block; and two separate constraints between the same pair of tables
  each get their own distinct edge rather than being collapsed into one.
- **Live-verified against a real, hand-built Postgres schema** in a
  scratch database created for this: `departments`, `employees` (with a
  real `department_id` FK *and* a real self-referencing `manager_id` FK
  to its own `id`), `projects`, and `employee_projects` as a genuine
  many-to-many junction table with a composite primary key and two
  foreign keys. Ran the actual compiled binary against this real
  database and confirmed all four real constraints appeared correctly
  in the generated DOT — including the self-loop
  (`"employees" -> "employees"`) and both junction-table edges — with
  the exact real column names pulled from `pg_attribute`, not guessed.

**Not done / deliberately deferred**: rendering to an actual image
(PNG/SVG) — this emits DOT text only; piping it through a real `dot`
binary (Graphviz wasn't installed in this sandbox, so that specific
rendering step itself wasn't exercised, only the text-generation logic
that feeds it) is left to the caller, matching how every other DOT-
emitting tool in this workspace (`depgraph`, `gitbloat`) already works.
Primary-key/unique-constraint annotations on nodes aren't shown, only
foreign-key edges. Views and materialized views aren't included, only
real tables (`pg_tables`).
