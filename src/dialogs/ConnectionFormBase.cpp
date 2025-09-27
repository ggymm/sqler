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
    layout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg);
    layout->setSpacing(Theme::Spacing::lg);

    m_formLayout = new QFormLayout();
    m_formLayout->setSpacing(Theme::Spacing::md);
    layout->addLayout(m_formLayout);

    // setupUI(); // Remove this - will be called by subclasses

    layout->addStretch();

    auto* buttonLayout = new QHBoxLayout();
    buttonLayout->setSpacing(Theme::Spacing::md);

    m_testButton = new QPushButton("测试连接", this);
    m_testButton->setObjectName("testButton");
    connect(m_testButton, &QPushButton::clicked, this, &ConnectionFormBase::onTestConnection);
    buttonLayout->addWidget(m_testButton);

    buttonLayout->addStretch();

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

    QString styleSheet = QString(
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
        "}"
        "QLineEdit:focus, QSpinBox:focus {"
        "    border-color: %5;"
        "}"
        "QLabel {"
        "    color: %4;"
        "    font-size: 14px;"
        "}"
        "QPushButton#testButton {"
        "    background-color: %5;"
        "    color: white;"
        "    border: none;"
        "    border-radius: %3px;"
        "    padding: 8px 16px;"
        "    font-size: 14px;"
        "}"
        "QPushButton#testButton:hover {"
        "    background-color: %6;"
        "}"
        "QPushButton#saveButton {"
        "    background-color: %5;"
        "    color: white;"
        "    border: none;"
        "    border-radius: %3px;"
        "    padding: 8px 16px;"
        "    font-size: 14px;"
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