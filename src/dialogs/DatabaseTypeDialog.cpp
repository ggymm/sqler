#include "DatabaseTypeDialog.h"
#include "../components/ThemedScrollArea.h"
#include "../components/ThemedButton.h"
#include "../Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QIcon>

DatabaseTypeDialog::DatabaseTypeDialog(QWidget* parent)
    : ThemedDialog(parent) {
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

    // No need to connect theme signals as ThemedDialog handles it
}

void DatabaseTypeDialog::setupUI() {
    // Don't set window title, let parent dialog handle it
    resize(500, 400);

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    // Create scroll area for database types
    auto* scrollArea = new ThemedScrollArea(this);

    auto* scrollWidget = new QWidget();
    scrollWidget->setObjectName("scrollContent");
    auto* scrollLayout = new QVBoxLayout(scrollWidget);
    scrollLayout->setSpacing(Theme::Spacing::sm);
    scrollLayout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg);

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

ThemedButton* DatabaseTypeDialog::createDatabaseTypeButton(const DatabaseType& dbType) {
    auto* button = new ThemedButton(this);
    button->setVariant(ThemedButton::Variant::Dialog);
    button->setFixedHeight(Theme::Sizes::dialogButtonHeight);

    auto* layout = new QHBoxLayout(button);
    layout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::md, Theme::Spacing::lg, Theme::Spacing::md);
    layout->setSpacing(Theme::Spacing::lg);

    // Icon on the left
    auto* iconLabel = new QLabel(button);
    iconLabel->setPixmap(QIcon(dbType.iconPath).pixmap(32, 32));
    iconLabel->setFixedSize(40, 40);
    iconLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(iconLabel);

    // Text content on the right
    auto* textLayout = new QVBoxLayout();
    textLayout->setSpacing(Theme::Spacing::xs);
    textLayout->setContentsMargins(0, 0, 0, 0);

    auto* nameLabel = new QLabel(dbType.displayName, button);
    nameLabel->setProperty("class", "subtitle");
    nameLabel->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
    textLayout->addWidget(nameLabel);

    auto* descLabel = new QLabel(dbType.description, button);
    descLabel->setProperty("class", "description");
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