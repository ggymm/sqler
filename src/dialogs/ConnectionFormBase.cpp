#include "ConnectionFormBase.h"
#include "../Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFormLayout>
#include <QPushButton>
#include <QSpacerItem>

ConnectionFormBase::ConnectionFormBase(QWidget* parent)
    : QWidget(parent)
    , m_formLayout(nullptr)
    , m_testButton(nullptr)
    , m_saveButton(nullptr) {

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::sm);
    layout->setSpacing(Theme::Spacing::lg);

    m_formLayout = new QFormLayout();
    m_formLayout->setSpacing(Theme::Spacing::md);
    m_formLayout->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    m_formLayout->setFormAlignment(Qt::AlignTop | Qt::AlignLeft);
    m_formLayout->setLabelAlignment(Qt::AlignRight | Qt::AlignVCenter);
    layout->addLayout(m_formLayout);

    layout->addStretch();

    // Button layout: test connection on left, other buttons on right
    auto* buttonLayout = new QHBoxLayout();
    buttonLayout->setSpacing(Theme::Spacing::md);

    // Left side: Test connection button
    m_testButton = new QPushButton("测试连接", this);
    m_testButton->setObjectName("testButton");
    connect(m_testButton, &QPushButton::clicked, this, &ConnectionFormBase::onTestConnection);
    buttonLayout->addWidget(m_testButton);

    buttonLayout->addStretch(); // Spacer between left and right buttons

    // Right side: Previous, Cancel, Save buttons
    m_backButton = new QPushButton("上一步", this);
    m_backButton->setObjectName("backButton");
    connect(m_backButton, &QPushButton::clicked, this, &ConnectionFormBase::backClicked);
    buttonLayout->addWidget(m_backButton);

    m_cancelButton = new QPushButton("取消", this);
    m_cancelButton->setObjectName("cancelButton");
    connect(m_cancelButton, &QPushButton::clicked, this, &ConnectionFormBase::cancelClicked);
    buttonLayout->addWidget(m_cancelButton);

    m_saveButton = new QPushButton("保存", this);
    m_saveButton->setObjectName("saveButton");
    connect(m_saveButton, &QPushButton::clicked, this, &ConnectionFormBase::onSaveConnection);
    buttonLayout->addWidget(m_saveButton);

    layout->addLayout(buttonLayout);

    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ConnectionFormBase::onThemeChanged);
}

void ConnectionFormBase::applyTheme() {
    const auto& colors = Theme::instance().colors();

    const QString styleSheet = QStringLiteral(
        "ConnectionFormBase {"
        "    background-color: %1;"
        "}"
        "QLineEdit, QSpinBox {"
        "    background-color: %1;"
        "    border: 1px solid %2;"
        "    border-radius: %3px;"
        "    padding: 8px;"
        "    color: %4;"
        "    font-size: 14px;"
        "    min-height: 20px;"
        "}"
        "QLineEdit:focus, QSpinBox:focus {"
        "    border-color: %5;"
        "}"
        "QLabel {"
        "    color: %4;"
        "    font-size: 14px;"
        "    font-weight: 500;"
        "}"
        "QPushButton#testButton {"
        "    background-color: %5;"
        "    color: white;"
        "    border: none;"
        "    border-radius: %3px;"
        "    padding: 8px 16px;"
        "    font-size: 14px;"
        "    min-width: 80px;"
        "}"
        "QPushButton#testButton:hover {"
        "    background-color: %6;"
        "}"
        "QPushButton#backButton, QPushButton#cancelButton {"
        "    background-color: transparent;"
        "    color: %4;"
        "    border: 1px solid %2;"
        "    border-radius: %3px;"
        "    padding: 8px 16px;"
        "    font-size: 14px;"
        "    min-width: 80px;"
        "}"
        "QPushButton#backButton:hover, QPushButton#cancelButton:hover {"
        "    background-color: %2;"
        "}"
        "QPushButton#saveButton {"
        "    background-color: %5;"
        "    color: white;"
        "    border: none;"
        "    border-radius: %3px;"
        "    padding: 8px 16px;"
        "    font-size: 14px;"
        "    min-width: 80px;"
        "}"
        "QPushButton#saveButton:hover {"
        "    background-color: %6;"
        "}"
    ).arg(colors.background.name())
     .arg(colors.border.name())
     .arg(Theme::Sizes::borderRadius)
     .arg(colors.text.name())
     .arg(colors.primary.name())
     .arg(colors.primaryHover.name());

    setStyleSheet(styleSheet);
}

void ConnectionFormBase::onThemeChanged() {
    applyTheme();
}

void ConnectionFormBase::onTestConnection() {
    // TODO: Implement connection testing
}

void ConnectionFormBase::onSaveConnection() {
    if (validateInput()) {
        emit connectionSaved();
    }
}