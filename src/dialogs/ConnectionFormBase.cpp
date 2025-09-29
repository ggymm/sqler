#include "ConnectionFormBase.h"

#include "../components/GPushButton.h"
#include "../components/GStyle.h"

#include <QFormLayout>
#include <QLabel>
#include <QShowEvent>
#include <QTimer>

ConnectionFormBase::ConnectionFormBase(QWidget* parent) : QWidget(parent), m_formLayout(nullptr), m_testButton(nullptr), m_saveButton(nullptr) {
    setAttribute(Qt::WA_StyledBackground, true);
    setObjectName("connectionFormPage");

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg);
    layout->setSpacing(GStyle::Spacing::lg);

    m_formLayout = new QFormLayout();
    m_formLayout->setSpacing(GStyle::Spacing::md);
    m_formLayout->setFieldGrowthPolicy(QFormLayout::AllNonFixedFieldsGrow);
    m_formLayout->setFormAlignment(Qt::AlignTop | Qt::AlignLeft);
    m_formLayout->setLabelAlignment(Qt::AlignRight | Qt::AlignVCenter);
    layout->addLayout(m_formLayout);

    layout->addStretch();

    // Footer area container to ensure dark header/footer vs light content
    m_footerWidget = new QWidget(this);
    m_footerWidget->setObjectName("dialogFooter");
    auto* buttonLayout = new QHBoxLayout(m_footerWidget);
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

    layout->addWidget(m_footerWidget);
}

void ConnectionFormBase::attachFooterTo(QHBoxLayout* destLayout) {
    if (!destLayout)
        return;

    // Hide local footer to avoid duplicate area inside the form
    if (m_footerWidget)
        m_footerWidget->setVisible(false);

    // Clear destination layout before populating
    while (QLayoutItem* item = destLayout->takeAt(0)) {
        if (auto* w = item->widget()) {
            w->setParent(nullptr);
        }
        delete item;
    }

    // Rebuild footer: left test, stretch, right actions
    if (m_testButton)
        destLayout->addWidget(m_testButton);
    destLayout->addStretch();
    if (m_backButton)
        destLayout->addWidget(m_backButton);
    if (m_cancelButton)
        destLayout->addWidget(m_cancelButton);
    if (m_saveButton)
        destLayout->addWidget(m_saveButton);
}

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

void ConnectionFormBase::showEvent(QShowEvent* event) {
    QWidget::showEvent(event);
    if (!m_formLayout)
        return;

    for (int row = 0; row < m_formLayout->rowCount(); ++row) {
        auto* labelItem = m_formLayout->itemAt(row, QFormLayout::LabelRole);
        auto* fieldItem = m_formLayout->itemAt(row, QFormLayout::FieldRole);
        if (!labelItem || !fieldItem)
            continue;

        QWidget* labelW = labelItem->widget();
        QWidget* fieldW = fieldItem->widget();
        if (!labelW || !fieldW)
            continue;

        if (auto* lbl = qobject_cast<QLabel*>(labelW)) {
            lbl->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
            const int h = qMax(fieldW->sizeHint().height(), fieldW->minimumSizeHint().height());
            if (h > 0) {
                lbl->setMinimumHeight(h);
            }
        }
    }
}
