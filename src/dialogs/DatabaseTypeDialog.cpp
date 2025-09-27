#include "DatabaseTypeDialog.h"
#include "../Theme.h"
#include <QVBoxLayout>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QPushButton>
#include <QLabel>
#include <QIcon>

DatabaseTypeDialog::DatabaseTypeDialog(QWidget* parent)
    : QDialog(parent) {
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
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &DatabaseTypeDialog::onThemeChanged);
}

void DatabaseTypeDialog::setupUI() {
    setWindowTitle("选择数据库类型");
    setModal(true);
    resize(600, 500);

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg);
    layout->setSpacing(Theme::Spacing::lg);

    auto* titleLabel = new QLabel("选择数据库类型", this);
    titleLabel->setObjectName("titleLabel");
    layout->addWidget(titleLabel);

    m_gridLayout = new QGridLayout();
    m_gridLayout->setSpacing(Theme::Spacing::md);

    int row = 0, col = 0;
    for (const auto& dbType : m_databaseTypes) {
        auto* button = createDatabaseTypeButton(dbType);
        m_gridLayout->addWidget(button, row, col);

        col++;
        if (col >= 2) {
            col = 0;
            row++;
        }
    }

    layout->addLayout(m_gridLayout);
    layout->addStretch();

    auto* buttonLayout = new QHBoxLayout();
    buttonLayout->addStretch();

    auto* cancelButton = new QPushButton("取消", this);
    cancelButton->setObjectName("cancelButton");
    connect(cancelButton, &QPushButton::clicked, this, &QDialog::reject);
    buttonLayout->addWidget(cancelButton);

    layout->addLayout(buttonLayout);
}

QPushButton* DatabaseTypeDialog::createDatabaseTypeButton(const DatabaseType& dbType) {
    auto* button = new QPushButton(this);
    button->setFixedSize(250, 100);
    button->setObjectName("databaseTypeButton");

    auto* layout = new QHBoxLayout(button);
    layout->setContentsMargins(Theme::Spacing::md, Theme::Spacing::md, Theme::Spacing::md, Theme::Spacing::md);
    layout->setSpacing(Theme::Spacing::md);

    auto* iconLabel = new QLabel(button);
    iconLabel->setPixmap(QIcon(dbType.iconPath).pixmap(32, 32));
    layout->addWidget(iconLabel);

    auto* textLayout = new QVBoxLayout();
    textLayout->setSpacing(Theme::Spacing::xs);

    auto* nameLabel = new QLabel(dbType.displayName, button);
    nameLabel->setObjectName("databaseName");
    textLayout->addWidget(nameLabel);

    auto* descLabel = new QLabel(dbType.description, button);
    descLabel->setObjectName("databaseDescription");
    descLabel->setWordWrap(true);
    textLayout->addWidget(descLabel);

    layout->addLayout(textLayout);

    connect(button, &QPushButton::clicked, [this, dbType]() {
        onDatabaseTypeSelected(dbType.id);
    });

    return button;
}

void DatabaseTypeDialog::applyTheme() {
    const auto& colors = Theme::instance().colors();

    QString styleSheet = QString(
        "DatabaseTypeDialog {"
        "    background-color: %1;"
        "}"
        "QLabel#titleLabel {"
        "    color: %2;"
        "    font-size: 16px;"
        "    font-weight: bold;"
        "}"
        "QPushButton#databaseTypeButton {"
        "    background-color: %1;"
        "    border: 1px solid %3;"
        "    border-radius: %4px;"
        "    text-align: left;"
        "}"
        "QPushButton#databaseTypeButton:hover {"
        "    background-color: %5;"
        "}"
        "QLabel#databaseName {"
        "    color: %2;"
        "    font-size: 14px;"
        "    font-weight: bold;"
        "}"
        "QLabel#databaseDescription {"
        "    color: %6;"
        "    font-size: 12px;"
        "}"
        "QPushButton#cancelButton {"
        "    color: %2;"
        "    background-color: transparent;"
        "    border: 1px solid %3;"
        "    border-radius: %4px;"
        "    padding: 8px 16px;"
        "}"
        "QPushButton#cancelButton:hover {"
        "    background-color: %5;"
        "}"
    ).arg(colors.background.name())
     .arg(colors.text.name())
     .arg(colors.border.name())
     .arg(Theme::Sizes::borderRadius)
     .arg(colors.surface.name())
     .arg(colors.textSecondary.name());

    setStyleSheet(styleSheet);
}

void DatabaseTypeDialog::onThemeChanged() {
    applyTheme();
}

void DatabaseTypeDialog::onDatabaseTypeSelected(const QString& type) {
    m_selectedType = type;
    accept();
}