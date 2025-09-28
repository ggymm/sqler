#include "ConnectionFormBase.h"
#include "../components/GStyle.h"
#include "../components/GPushButton.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFormLayout>
#include <QSpacerItem>
#include <QTimer>

ConnectionFormBase::ConnectionFormBase(QWidget* parent)
    : QWidget(parent)
    , m_formLayout(nullptr)
    , m_testButton(nullptr)
    , m_saveButton(nullptr) {

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::sm);
    layout->setSpacing(GStyle::Spacing::lg);

    m_formLayout = new QFormLayout();
    m_formLayout->setSpacing(GStyle::Spacing::md);
    m_formLayout->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    m_formLayout->setFormAlignment(Qt::AlignTop | Qt::AlignLeft);
    m_formLayout->setLabelAlignment(Qt::AlignRight | Qt::AlignVCenter);
    layout->addLayout(m_formLayout);

    layout->addStretch();

    // Button layout: test connection on left, other buttons on right
    auto* buttonLayout = new QHBoxLayout();
    buttonLayout->setSpacing(GStyle::Spacing::md);

    // Left side: Test connection button
    m_testButton = new GPushButton("测试连接", GPushButton::Variant::Primary, this);
    connect(m_testButton, &QPushButton::clicked, this, &ConnectionFormBase::onTestConnection);
    buttonLayout->addWidget(m_testButton);

    buttonLayout->addStretch(); // Spacer between left and right buttons

    // Right side: Previous, Cancel, Save buttons
    m_backButton = new GPushButton("上一步", GPushButton::Variant::Secondary, this);
    connect(m_backButton, &QPushButton::clicked, this, &ConnectionFormBase::backClicked);
    buttonLayout->addWidget(m_backButton);

    m_cancelButton = new GPushButton("取消", GPushButton::Variant::Secondary, this);
    connect(m_cancelButton, &QPushButton::clicked, this, &ConnectionFormBase::cancelClicked);
    buttonLayout->addWidget(m_cancelButton);

    m_saveButton = new GPushButton("保存", GPushButton::Variant::Primary, this);
    connect(m_saveButton, &QPushButton::clicked, this, &ConnectionFormBase::onSaveConnection);
    buttonLayout->addWidget(m_saveButton);

    layout->addLayout(buttonLayout);

    applyTheme();
}

void ConnectionFormBase::applyTheme() { /* no page-level styles */ }

void ConnectionFormBase::onTestConnection() {
    m_testButton->setEnabled(false);
    m_testButton->setText("测试中...");

    // Validate input first
    if (!validateInput()) {
        m_testButton->setEnabled(true);
        m_testButton->setText("测试连接");
        return;
    }

    // Simulate connection test (in real implementation, would test actual connection)
    QTimer::singleShot(1000, this, [this]() {
        m_testButton->setText("连接成功");
        QTimer::singleShot(1500, this, [this]() {
            m_testButton->setEnabled(true);
            m_testButton->setText("测试连接");
        });
    });
}

void ConnectionFormBase::onSaveConnection() {
    if (validateInput()) {
        emit connectionSaved();
    }
}
