use super::*;

/// PostgreSQL 查询构建器
pub struct PostgreSQLBuilder;

impl QueryBuilder for PostgreSQLBuilder {
    fn build_where_clause(
        &self,
        conditions: &[FilterCondition],
    ) -> (String, Vec<String>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 0;

        for condition in conditions.iter() {
            let field = self.escape_identifier(&condition.field);

            match condition.operator {
                Operator::IsNull => {
                    clauses.push(format!("{} IS NULL", field));
                }
                Operator::IsNotNull => {
                    clauses.push(format!("{} IS NOT NULL", field));
                }
                Operator::In => {
                    if let ConditionValue::List(ref list) = condition.value {
                        if list.is_empty() {
                            continue;
                        }
                        let placeholders: Vec<_> = (0..list.len())
                            .map(|_| {
                                param_index += 1;
                                self.placeholder(param_index - 1)
                            })
                            .collect();
                        clauses.push(format!("{} IN ({})", field, placeholders.join(", ")));
                        params.extend(list.clone());
                    }
                }
                Operator::NotIn => {
                    if let ConditionValue::List(ref list) = condition.value {
                        if list.is_empty() {
                            continue;
                        }
                        let placeholders: Vec<_> = (0..list.len())
                            .map(|_| {
                                param_index += 1;
                                self.placeholder(param_index - 1)
                            })
                            .collect();
                        clauses.push(format!("{} NOT IN ({})", field, placeholders.join(", ")));
                        params.extend(list.clone());
                    }
                }
                Operator::Between => {
                    if let ConditionValue::Range(ref start, ref end) = condition.value {
                        param_index += 1;
                        let ph1 = self.placeholder(param_index - 1);
                        param_index += 1;
                        let ph2 = self.placeholder(param_index - 1);
                        clauses.push(format!("{} BETWEEN {} AND {}", field, ph1, ph2));
                        params.push(start.clone());
                        params.push(end.clone());
                    }
                }
                _ => {
                    let op_str = match condition.operator {
                        Operator::Equal => "=",
                        Operator::NotEqual => "!=",
                        Operator::GreaterThan => ">",
                        Operator::LessThan => "<",
                        Operator::GreaterOrEqual => ">=",
                        Operator::LessOrEqual => "<=",
                        Operator::Like => "LIKE",
                        Operator::NotLike => "NOT LIKE",
                        _ => "=",
                    };

                    param_index += 1;
                    let ph = self.placeholder(param_index - 1);
                    clauses.push(format!("{} {} {}", field, op_str, ph));

                    match &condition.value {
                        ConditionValue::String(s) => params.push(s.clone()),
                        ConditionValue::Number(n) => params.push(n.to_string()),
                        ConditionValue::Bool(b) => {
                            params.push(if *b { "true".to_string() } else { "false".to_string() })
                        }
                        _ => {}
                    }
                }
            }
        }

        if clauses.is_empty() {
            ("1=1".to_string(), params)
        } else {
            (clauses.join(" AND "), params)
        }
    }

    fn build_order_clause(
        &self,
        sorts: &[SortOrder],
    ) -> String {
        let orders: Vec<_> = sorts
            .iter()
            .map(|sort| {
                format!(
                    "{} {}",
                    self.escape_identifier(&sort.field),
                    if sort.ascending { "ASC" } else { "DESC" }
                )
            })
            .collect();

        if orders.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", orders.join(", "))
        }
    }

    fn build_limit_clause(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String {
        match (limit, offset) {
            (Some(l), Some(o)) => format!("LIMIT {} OFFSET {}", l, o),
            (Some(l), None) => format!("LIMIT {}", l),
            (None, Some(o)) => format!("OFFSET {}", o),
            (None, None) => String::new(),
        }
    }

    fn escape_identifier(
        &self,
        identifier: &str,
    ) -> String {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn placeholder(
        &self,
        index: usize,
    ) -> String {
        format!("${}", index + 1)
    }
}
