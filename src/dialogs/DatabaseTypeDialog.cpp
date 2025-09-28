#include "DatabaseTypeDialog.h"
#include "../components/GScrollArea.h"
#include "../components/GPushButton.h"
#include "../components/GLabel.h"
#include "../components/GStyle.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QIcon>

DatabaseTypeDialog::DatabaseTypeDialog(QWidget* parent)
    : GDialog(parent) {
    m_databaseTypes = {
        {"mysql", "MySQL", "流行的开源关系型数据库", ":/assets/icons/db/mysql.svg"},
        {"postgresql", "PostgreSQL", "高级开源关系型数据库", ":/assets/icons/db/postgresql.svg"},
        {"sqlite", "SQLite", "轻量级文件数据库", ":/assets/icons/db/sqlite.svg"},
        {"mongodb", "MongoDB", "文档型NoSQL数据库", ":/assets/icons/db/mongodb.svg"},
        {"redis", "Redis", "内存键值存储数据库", ":/assets/icons/db/redis.svg"},
        {"oracle", "Oracle", "企业级关系型数据库", ":/assets/icons/db/oracle.svg"},
        {"sqlserver", "SQL Server", "微软关系型数据库", ":/assets/icons/db/sqlserver.svg"}
    };

    setupUI();
}

void DatabaseTypeDialog::setupUI() {
    // Don't set window title, let parent dialog handle it
    resize(500, 400);

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    // Create scroll area for database types
    auto* scrollArea = new GScrollArea(this);

    auto* scrollWidget = new QWidget();
    scrollWidget->setObjectName("scrollContent");
    auto* scrollLayout = new QVBoxLayout(scrollWidget);
    scrollLayout->setSpacing(GStyle::Spacing::sm);
    scrollLayout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg);

    // Add database type items in vertical list
    for (const auto& dbType : m_databaseTypes) {
        auto* button = createDatabaseTypeButton(dbType);
        scrollLayout->addWidget(button);
    }

    scrollArea->setWidget(scrollWidget);
    layout->addWidget(scrollArea);

    // Remove the cancel button as NewConnectionDialog handles it
    // auto* buttonLayout = new QHBoxLayout();
    // buttonLayout->addStretch();
    // auto* cancelButton = new QPushButton("取消", this);
    // cancelButton->setObjectName("cancelButton");
    // connect(cancelButton, &QPushButton::clicked, this, &QDialog::reject);
    // buttonLayout->addWidget(cancelButton);
    // layout->addLayout(buttonLayout);
}

GPushButton* DatabaseTypeDialog::createDatabaseTypeButton(const DatabaseType& dbType) {
    auto* button = new GPushButton(this);
    button->setVariant(GPushButton::Variant::Dialog);
    button->setFixedHeight(GStyle::Sizes::dialogButtonHeight);

    auto* layout = new QHBoxLayout(button);
    layout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::md, GStyle::Spacing::lg, GStyle::Spacing::md);
    layout->setSpacing(GStyle::Spacing::lg);

    // Icon on the left
    auto* iconLabel = new GLabel(button);
    iconLabel->setPixmap(QIcon(dbType.iconPath).pixmap(32, 32));
    iconLabel->setFixedSize(40, 40);
    iconLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(iconLabel);

    // Text content on the right
    auto* textLayout = new QVBoxLayout();
    textLayout->setSpacing(GStyle::Spacing::xs);
    textLayout->setContentsMargins(0, 0, 0, 0);

    auto* nameLabel = new GLabel(dbType.displayName, GLabel::Role::Emphasis, button);
    nameLabel->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    textLayout->addWidget(nameLabel);

    auto* descLabel = new GLabel(dbType.description, GLabel::Role::Caption, button);
    descLabel->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    descLabel->setWordWrap(true);
    textLayout->addWidget(descLabel);

    layout->addLayout(textLayout);
    layout->addStretch(); // Push content to the left

    connect(button, &QPushButton::clicked, [this, dbType]() {
        onDatabaseTypeSelected(dbType.id);
    });

    return button;
}

void DatabaseTypeDialog::onDatabaseTypeSelected(const QString& type) {
    m_selectedType = type;
    accept();
}
