use iced::widget::{Rule, column, container, horizontal_space, row, text, vertical_space};
use iced::{Background, Element, Length, Shadow};

use crate::app::{Connection, ContentTab, Message, Palette};

pub fn render(
    tab: ContentTab,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    match tab {
        ContentTab::Tables => tables_view(connection, palette),
        ContentTab::Queries => queries_view(connection, palette),
        ContentTab::Functions => functions_view(connection, palette),
        ContentTab::Users => users_view(connection, palette),
    }
}

#[derive(Clone, Copy)]
struct TableInfo {
    name: &'static str,
    engine: &'static str,
    rows: u32,
    comment: &'static str,
    updated: &'static str,
}

const SAMPLE_TABLES: &[TableInfo] = &[
    TableInfo {
        name: "users",
        engine: "InnoDB",
        rows: 12430,
        comment: "注册用户主表",
        updated: "2024-05-18 21:46",
    },
    TableInfo {
        name: "orders",
        engine: "InnoDB",
        rows: 8731,
        comment: "订单记录，含状态与支付信息",
        updated: "2024-05-18 20:11",
    },
    TableInfo {
        name: "order_items",
        engine: "InnoDB",
        rows: 24108,
        comment: "订单明细项",
        updated: "2024-05-18 19:52",
    },
    TableInfo {
        name: "audit_logs",
        engine: "MyISAM",
        rows: 151209,
        comment: "系统审计与操作日志",
        updated: "2024-05-18 21:02",
    },
];

#[derive(Clone, Copy)]
struct ColumnInfo {
    name: &'static str,
    data_type: &'static str,
    nullable: bool,
    default: &'static str,
    comment: &'static str,
}

const USERS_COLUMNS: &[ColumnInfo] = &[
    ColumnInfo {
        name: "id",
        data_type: "BIGINT UNSIGNED",
        nullable: false,
        default: "AUTO_INCREMENT",
        comment: "主键",
    },
    ColumnInfo {
        name: "email",
        data_type: "VARCHAR(255)",
        nullable: false,
        default: "",
        comment: "登录邮箱，唯一",
    },
    ColumnInfo {
        name: "display_name",
        data_type: "VARCHAR(120)",
        nullable: false,
        default: "",
        comment: "昵称",
    },
    ColumnInfo {
        name: "status",
        data_type: "ENUM('active','pending','disabled')",
        nullable: false,
        default: "'pending'",
        comment: "账户状态",
    },
    ColumnInfo {
        name: "last_login_at",
        data_type: "DATETIME",
        nullable: true,
        default: "NULL",
        comment: "最近登录时间",
    },
];

#[derive(Clone, Copy)]
struct IndexInfo {
    name: &'static str,
    columns: &'static str,
    kind: &'static str,
    remark: &'static str,
}

const USERS_INDEXES: &[IndexInfo] = &[
    IndexInfo {
        name: "PRIMARY",
        columns: "id",
        kind: "PRIMARY",
        remark: "聚簇索引",
    },
    IndexInfo {
        name: "uniq_email",
        columns: "email",
        kind: "UNIQUE",
        remark: "确保邮箱唯一",
    },
    IndexInfo {
        name: "idx_status_login",
        columns: "status,last_login_at",
        kind: "BTREE",
        remark: "登录态筛选",
    },
];

fn tables_view(
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let summary = connection.summary();

    let mut list = column![
        text("核心数据表").size(16).color(palette.text).width(Length::Fill),
        text(format!("{} · {}", connection.name, summary))
            .size(13)
            .color(palette.text_muted),
        vertical_space().height(8),
    ]
    .spacing(12);

    for table in SAMPLE_TABLES {
        list = list.push(table_card(*table, palette));
    }

    let table_overview = container(list)
        .padding(20)
        .width(Length::FillPortion(2))
        .style(move |_| panel_style(palette));

    let detail = container(
        column![
            text("users 表结构预览").size(16).color(palette.text),
            text("聚焦用户主表字段与索引，快速了解数据形态。")
                .size(13)
                .color(palette.text_muted),
            vertical_space().height(12),
            columns_table(USERS_COLUMNS, palette),
            vertical_space().height(18),
            indexes_table(USERS_INDEXES, palette),
        ]
        .spacing(12),
    )
    .padding(20)
    .width(Length::FillPortion(3))
    .style(move |_| panel_style(palette));

    row![table_overview, detail].spacing(20).into()
}

fn table_card(
    info: TableInfo,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(info.name).size(17).color(palette.text),
                horizontal_space(),
                badge(info.engine, palette),
            ]
            .align_y(iced::Alignment::Center),
            text(info.comment)
                .size(13)
                .color(palette.text_muted)
                .width(Length::Fill),
            text(format!("{} 行 · 最近更新 {}", info.rows, info.updated))
                .size(12)
                .color(palette.text_muted),
        ]
        .spacing(8),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

fn columns_table(
    columns: &[ColumnInfo],
    palette: Palette,
) -> Element<'static, Message> {
    let mut rows = column![table_header(&["字段", "类型", "允许空值", "默认值", "备注"], palette)];
    rows = rows.push(Rule::horizontal(1).style(move |_| rule_style(palette)));

    for column in columns {
        rows = rows.push(
            container(
                row![
                    text(column.name).size(14).color(palette.text),
                    text(column.data_type)
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fixed(160.0)),
                    text(if column.nullable { "是" } else { "否" })
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fixed(80.0)),
                    text(column.default)
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fixed(120.0)),
                    text(column.comment)
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            )
            .padding([8, 0]),
        );
    }

    container(rows.spacing(6))
        .padding([12, 16])
        .style(move |_| card_style(palette))
        .into()
}

fn indexes_table(
    indexes: &[IndexInfo],
    palette: Palette,
) -> Element<'static, Message> {
    let mut rows = column![table_header(&["索引名", "列", "类型", "说明"], palette)];
    rows = rows.push(Rule::horizontal(1).style(move |_| rule_style(palette)));

    for index in indexes {
        rows = rows.push(
            container(
                row![
                    text(index.name).size(14).color(palette.text),
                    text(index.columns)
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fixed(180.0)),
                    badge(index.kind, palette),
                    text(index.remark)
                        .size(14)
                        .color(palette.text_muted)
                        .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            )
            .padding([8, 0]),
        );
    }

    container(rows.spacing(6))
        .padding([12, 16])
        .style(move |_| card_style(palette))
        .into()
}

fn queries_view(
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let summary = connection.summary();

    let editor_panel = container(
        column![
            text("MySQL 查询工作台").size(16).color(palette.text),
            text(format!("关联连接：{}", summary))
                .size(13)
                .color(palette.text_muted),
            vertical_space().height(12),
            container(
                column![
                    text("-- 活动 SQL 草稿").size(13).color(palette.text_muted),
                    vertical_space().height(4),
                    text("SELECT u.id, u.email, COUNT(o.id) AS order_count")
                        .size(15)
                        .color(palette.text),
                    text("FROM users u").size(15).color(palette.text),
                    text("LEFT JOIN orders o ON o.user_id = u.id")
                        .size(15)
                        .color(palette.text),
                    text("WHERE u.status = 'active'").size(15).color(palette.text),
                    text("GROUP BY u.id, u.email").size(15).color(palette.text),
                    text("ORDER BY order_count DESC").size(15).color(palette.text),
                ]
                .spacing(2),
            )
            .padding(16)
            .style(move |_| editor_style(palette)),
            vertical_space().height(12),
            text("提示：SQL 执行结果将在右侧结果面板展示，并支持导出与复制。")
                .size(13)
                .color(palette.text_muted),
        ]
        .spacing(8),
    )
    .padding(20)
    .style(move |_| panel_style(palette));

    let mut history = column![
        text("近期执行历史").size(16).color(palette.text),
        vertical_space().height(8),
    ];

    let samples = [
        ("2024-05-18 21:32", "统计活跃用户下单数", "耗时 124 ms · 返回 128 行"),
        ("2024-05-18 20:05", "更新订单状态为发货", "耗时 42 ms · 影响 37 行"),
        ("2024-05-18 18:22", "拉取近 7 日错误日志", "耗时 312 ms · 返回 521 行"),
    ];

    for (time, title, meta) in samples {
        history = history.push(
            container(
                column![
                    row![
                        text(title).size(15).color(palette.text),
                        horizontal_space(),
                        text(time).size(12).color(palette.text_muted),
                    ],
                    text(meta).size(12).color(palette.text_muted),
                ]
                .spacing(4),
            )
            .padding(14)
            .style(move |_| card_style(palette)),
        );
    }

    let history_panel = container(history.spacing(10))
        .padding(20)
        .style(move |_| panel_style(palette));

    column![editor_panel, vertical_space().height(16), history_panel]
        .spacing(0)
        .into()
}

#[derive(Clone, Copy)]
struct Routine {
    name: &'static str,
    routine_type: &'static str,
    returns: &'static str,
    summary: &'static str,
}

const SAMPLE_ROUTINES: &[Routine] = &[
    Routine {
        name: "fn_recent_orders",
        routine_type: "FUNCTION",
        returns: "JSON",
        summary: "返回指定用户近 30 日订单概要。",
    },
    Routine {
        name: "sp_rebuild_daily_stats",
        routine_type: "PROCEDURE",
        returns: "VOID",
        summary: "重建统计表，聚合订单与支付指标。",
    },
    Routine {
        name: "tr_orders_set_status",
        routine_type: "TRIGGER",
        returns: "AFTER INSERT",
        summary: "新建订单时自动写入操作日志。",
    },
];

fn functions_view(
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let mut routines = column![
        text(format!("{} 内的函数与存储过程", connection.name))
            .size(16)
            .color(palette.text),
        text("集中管理业务函数、存储过程与触发器，支持版本追踪与发布。")
            .size(13)
            .color(palette.text_muted),
        vertical_space().height(12),
    ];

    for routine in SAMPLE_ROUTINES {
        routines = routines.push(routine_card(*routine, palette));
    }

    let best_practices = container(
        column![
            text("最佳实践").size(15).color(palette.text),
            vertical_space().height(6),
            text("• 通过 Git 同步例程脚本，保障流程可审计。")
                .size(13)
                .color(palette.text_muted),
            text("• 提交前在沙箱库执行，确保兼容性与权限。")
                .size(13)
                .color(palette.text_muted),
            text("• 使用标签标记生产可执行版本，降低回滚成本。")
                .size(13)
                .color(palette.text_muted),
        ]
        .spacing(4),
    )
    .padding(20)
    .style(move |_| panel_style(palette));

    column![
        container(routines.spacing(12))
            .padding(20)
            .style(move |_| panel_style(palette)),
        vertical_space().height(16),
        best_practices,
    ]
    .spacing(0)
    .into()
}

fn routine_card(
    routine: Routine,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(routine.name).size(16).color(palette.text),
                horizontal_space(),
                badge(routine.routine_type, palette),
                badge(routine.returns, palette),
            ]
            .align_y(iced::Alignment::Center),
            text(routine.summary)
                .size(13)
                .color(palette.text_muted)
                .width(Length::Fill),
        ]
        .spacing(6),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

#[derive(Clone, Copy)]
struct MysqlUser {
    user: &'static str,
    host: &'static str,
    roles: &'static str,
    last_seen: &'static str,
    remark: &'static str,
}

const SAMPLE_USERS: &[MysqlUser] = &[
    MysqlUser {
        user: "admin",
        host: "%",
        roles: "DBA, Replication",
        last_seen: "2024-05-18 21:47",
        remark: "平台管理员，具备全库权限",
    },
    MysqlUser {
        user: "readonly_app",
        host: "10.0.%._",
        roles: "Reader",
        last_seen: "2024-05-18 20:58",
        remark: "线上业务查询账号，仅 SELECT",
    },
    MysqlUser {
        user: "etl_job",
        host: "172.16.%._",
        roles: "Writer, Event",
        last_seen: "2024-05-18 03:16",
        remark: "离线同步任务用户",
    },
];

fn users_view(
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let mut entries = column![
        text(format!("{} 用户与权限", connection.name))
            .size(16)
            .color(palette.text),
        text("定位 MySQL 账户与角色映射，配合权限模板快速配置。")
            .size(13)
            .color(palette.text_muted),
        vertical_space().height(12),
    ];

    for user in SAMPLE_USERS {
        entries = entries.push(user_card(*user, palette));
    }

    let recommendations = container(
        column![
            text("权限建议").size(15).color(palette.text),
            vertical_space().height(6),
            text("• 生产环境使用最小权限原则，并绑定独立角色。")
                .size(13)
                .color(palette.text_muted),
            text("• 定期轮换高敏用户密码，并开启登录审计。")
                .size(13)
                .color(palette.text_muted),
            text("• 配置只读副本账号，隔离分析类查询。")
                .size(13)
                .color(palette.text_muted),
        ]
        .spacing(4),
    )
    .padding(20)
    .style(move |_| panel_style(palette));

    column![
        container(entries.spacing(12))
            .padding(20)
            .style(move |_| panel_style(palette)),
        vertical_space().height(16),
        recommendations,
    ]
    .spacing(0)
    .into()
}

fn user_card(
    user: MysqlUser,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(format!("{}@{}", user.user, user.host))
                    .size(16)
                    .color(palette.text),
                horizontal_space(),
                badge(user.roles, palette),
            ]
            .align_y(iced::Alignment::Center),
            row![
                text(format!("最近访问：{}", user.last_seen))
                    .size(12)
                    .color(palette.text_muted),
                horizontal_space(),
                text(user.remark).size(12).color(palette.text_muted),
            ],
        ]
        .spacing(6),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

fn table_header(
    titles: &[&str],
    palette: Palette,
) -> Element<'static, Message> {
    let mut header = row![];
    for title in titles {
        header = header.push(
            text(title.to_string())
                .size(13)
                .color(palette.text)
                .width(Length::FillPortion(1)),
        );
    }

    container(header.spacing(12).align_y(iced::Alignment::Center)).into()
}

fn badge(
    label: &str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(label.to_string()).size(12).color(palette.accent))
        .padding([4, 10])
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.accent_soft)),
            text_color: Some(palette.accent),
            border: iced::border::Border {
                color: palette.accent,
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

fn panel_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.surface_muted)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn card_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn editor_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.background)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn rule_style(palette: Palette) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: palette.border,
        width: 1,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
    }
}
