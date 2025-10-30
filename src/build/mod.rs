/// 通用的筛选操作符
#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    Between,
    IsNull,
    IsNotNull,
}

impl Operator {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::LessThan,
            Self::GreaterOrEqual,
            Self::LessOrEqual,
            Self::Like,
            Self::NotLike,
            Self::IsNull,
            Self::IsNotNull,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::Between => "BETWEEN",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "=" => Self::Equal,
            "!=" => Self::NotEqual,
            ">" => Self::GreaterThan,
            "<" => Self::LessThan,
            ">=" => Self::GreaterOrEqual,
            "<=" => Self::LessOrEqual,
            "LIKE" => Self::Like,
            "NOT LIKE" => Self::NotLike,
            "IS NULL" => Self::IsNull,
            "IS NOT NULL" => Self::IsNotNull,
            "IN" => Self::In,
            "NOT IN" => Self::NotIn,
            "BETWEEN" => Self::Between,
            _ => Self::Equal, // 默认
        }
    }
}

/// 筛选条件的值
#[derive(Clone, Debug)]
pub enum ConditionValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    List(Vec<String>),     // 用于 IN
    Range(String, String), // 用于 BETWEEN
}

/// 单个筛选条件
#[derive(Clone, Debug)]
pub struct FilterCondition {
    pub field: String,
    pub operator: Operator,
    pub value: ConditionValue,
}

/// 排序规则
#[derive(Clone, Debug)]
pub struct SortOrder {
    pub field: String,
    pub ascending: bool,
}

/// 完整的查询条件
#[derive(Clone, Debug, Default)]
pub struct QueryConditions {
    pub filters: Vec<FilterCondition>,
    pub sorts: Vec<SortOrder>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 数据库类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLite,
    Oracle,
    SQLServer,
}

/// 通用查询构建器 trait
pub trait QueryBuilder {
    /// 构建 WHERE 子句
    fn build_where_clause(
        &self,
        conditions: &[FilterCondition],
    ) -> (String, Vec<String>);

    /// 构建 ORDER BY 子句
    fn build_order_clause(
        &self,
        sorts: &[SortOrder],
    ) -> String;

    /// 构建 LIMIT/OFFSET 子句
    fn build_limit_clause(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String;

    /// 转义标识符（表名、列名）
    fn escape_identifier(
        &self,
        identifier: &str,
    ) -> String;

    /// 获取参数占位符
    fn placeholder(
        &self,
        index: usize,
    ) -> String;

    /// 构建完整的 SELECT 查询
    fn build_select_query(
        &self,
        table: &str,
        columns: &[&str],
        conditions: &QueryConditions,
    ) -> (String, Vec<String>) {
        let cols = if columns.is_empty() {
            "*".to_string()
        } else {
            columns
                .iter()
                .map(|c| self.escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols, self.escape_identifier(table));

        let mut params = Vec::new();

        // WHERE 子句
        if !conditions.filters.is_empty() {
            let (where_clause, where_params) = self.build_where_clause(&conditions.filters);
            sql.push_str(&format!(" WHERE {}", where_clause));
            params.extend(where_params);
        }

        // ORDER BY 子句
        if !conditions.sorts.is_empty() {
            let order_clause = self.build_order_clause(&conditions.sorts);
            if !order_clause.is_empty() {
                sql.push_str(&format!(" {}", order_clause));
            }
        }

        // LIMIT/OFFSET 子句
        let limit_clause = self.build_limit_clause(conditions.limit, conditions.offset);
        if !limit_clause.is_empty() {
            sql.push_str(&format!(" {}", limit_clause));
        }

        (sql, params)
    }

    /// 构建 COUNT 查询
    fn build_count_query(
        &self,
        table: &str,
        conditions: &QueryConditions,
    ) -> (String, Vec<String>) {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.escape_identifier(table));
        let mut params = Vec::new();

        // WHERE 子句
        if !conditions.filters.is_empty() {
            let (where_clause, where_params) = self.build_where_clause(&conditions.filters);
            sql.push_str(&format!(" WHERE {}", where_clause));
            params.extend(where_params);
        }

        (sql, params)
    }
}

mod mysql;
mod postgres;
mod sqlite;

// 重新导出各数据库的实现
pub use mysql::MySQLBuilder;
pub use postgres::PostgreSQLBuilder;
pub use sqlite::SQLiteBuilder;

/// 根据数据库类型创建对应的查询构建器
pub fn create_builder(db_type: DatabaseType) -> Box<dyn QueryBuilder> {
    match db_type {
        DatabaseType::MySQL => Box::new(MySQLBuilder),
        DatabaseType::PostgreSQL => Box::new(PostgreSQLBuilder),
        DatabaseType::SQLite => Box::new(MySQLBuilder), // SQLite 使用类似 MySQL 的语法
        _ => Box::new(MySQLBuilder),                    // 默认使用 MySQL
    }
}
