//! Pure rendering: a list of real foreign-key relationships to
//! Graphviz DOT text. No database access here — `main.rs` is the only
//! part that talks to Postgres.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FkRelation {
    pub constraint_name: String,
    pub from_table: String,
    pub from_columns: Vec<String>,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

/// Renders every relation as a `digraph`, one node per distinct table
/// (including a table with no FKs at all, so an isolated table still
/// shows up rather than silently vanishing) and one edge per
/// constraint, labeled with the real column mapping.
pub fn render_dot(tables: &[String], relations: &[FkRelation]) -> String {
    let mut out = String::from("digraph pgerd {\n");
    out.push_str("  node [shape=box];\n");
    for table in tables {
        out.push_str(&format!("  \"{table}\";\n"));
    }
    for rel in relations {
        let from_cols = rel.from_columns.join(",");
        let to_cols = rel.to_columns.join(",");
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{} ({}) -> ({})\"];\n",
            rel.from_table, rel.to_table, rel.constraint_name, from_cols, to_cols
        ));
    }
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(name: &str, from: &str, from_cols: &[&str], to: &str, to_cols: &[&str]) -> FkRelation {
        FkRelation {
            constraint_name: name.to_string(),
            from_table: from.to_string(),
            from_columns: from_cols.iter().map(|s| s.to_string()).collect(),
            to_table: to.to_string(),
            to_columns: to_cols.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn renders_a_single_relation_as_one_edge() {
        let tables = vec!["orders".to_string(), "customers".to_string()];
        let rels = vec![rel(
            "orders_customer_id_fkey",
            "orders",
            &["customer_id"],
            "customers",
            &["id"],
        )];
        let dot = render_dot(&tables, &rels);
        assert!(dot.contains("\"orders\" -> \"customers\""));
        assert!(dot.contains("orders_customer_id_fkey"));
        assert!(dot.contains("(customer_id) -> (id)"));
    }

    #[test]
    fn a_table_with_no_fks_still_appears_as_an_isolated_node() {
        let tables = vec!["standalone".to_string()];
        let dot = render_dot(&tables, &[]);
        assert!(dot.contains("\"standalone\";"));
    }

    #[test]
    fn self_referencing_fk_renders_as_a_self_loop() {
        let tables = vec!["employees".to_string()];
        let rels = vec![rel(
            "employees_manager_id_fkey",
            "employees",
            &["manager_id"],
            "employees",
            &["id"],
        )];
        let dot = render_dot(&tables, &rels);
        assert!(dot.contains("\"employees\" -> \"employees\""));
    }

    #[test]
    fn composite_key_relation_joins_columns_with_commas() {
        let tables = vec!["a".to_string(), "b".to_string()];
        let rels = vec![rel(
            "a_b_fkey",
            "a",
            &["b_id1", "b_id2"],
            "b",
            &["id1", "id2"],
        )];
        let dot = render_dot(&tables, &rels);
        assert!(dot.contains("(b_id1,b_id2) -> (id1,id2)"));
    }

    #[test]
    fn output_is_a_well_formed_digraph_block() {
        let dot = render_dot(&["t".to_string()], &[]);
        assert!(dot.starts_with("digraph pgerd {\n"));
        assert!(dot.trim_end().ends_with('}'));
    }

    #[test]
    fn multiple_relations_between_the_same_two_tables_each_get_their_own_edge() {
        let tables = vec!["a".to_string(), "b".to_string()];
        let rels = vec![
            rel("a_b1_fkey", "a", &["b1_id"], "b", &["id"]),
            rel("a_b2_fkey", "a", &["b2_id"], "b", &["id"]),
        ];
        let dot = render_dot(&tables, &rels);
        assert_eq!(dot.matches("\"a\" -> \"b\"").count(), 2);
    }
}
