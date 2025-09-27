#include "ThemedFormWidget.h"
#include "ThemedButton.h"
#include "../Theme.h"
#include <QFormLayout>
#include <QVBoxLayout>
#include <QHBoxLayout>

ThemedFormWidget::ThemedFormWidget(QWidget* parent)
    : QWidget(parent)
    , m_formLayout(nullptr)
    , m_buttonContainer(nullptr) {
    setupForm();
}

void ThemedFormWidget::setupForm() {
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

    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedFormWidget::onThemeChanged);
}

void ThemedFormWidget::addFormRow(const QString& label, QWidget* field) {
    m_formLayout->addRow(label, field);
}

void ThemedFormWidget::addFormButtons(const QStringList& leftButtons, const QStringList& rightButtons) {
    if (m_buttonContainer) {
        m_buttonContainer->deleteLater();
        m_buttons.clear();
    }

    m_buttonContainer = new QWidget(this);
    auto* buttonLayout = new QHBoxLayout(m_buttonContainer);
    buttonLayout->setContentsMargins(0, Theme::Spacing::md, 0, 0);
    buttonLayout->setSpacing(Theme::Spacing::md);

    // Add left buttons
    for (const QString& buttonText : leftButtons) {
        auto* button = new ThemedButton(buttonText, ThemedButton::Variant::Primary, m_buttonContainer);
        connect(button, &QPushButton::clicked, this, &ThemedFormWidget::onButtonClicked);
        buttonLayout->addWidget(button);
        m_buttons.append(button);
    }

    buttonLayout->addStretch();

    // Add right buttons
    for (const QString& buttonText : rightButtons) {
        auto* button = new ThemedButton(buttonText, ThemedButton::Variant::Secondary, m_buttonContainer);
        connect(button, &QPushButton::clicked, this, &ThemedFormWidget::onButtonClicked);
        buttonLayout->addWidget(button);
        m_buttons.append(button);
    }

    layout()->addWidget(m_buttonContainer);
}

ThemedButton* ThemedFormWidget::getButton(const QString& text) const {
    for (auto* button : m_buttons) {
        if (button->text() == text) {
            return button;
        }
    }
    return nullptr;
}

void ThemedFormWidget::onThemeChanged() {
    applyTheme();
}

void ThemedFormWidget::onButtonClicked() {
    if (auto* button = qobject_cast<ThemedButton*>(sender())) {
        emit buttonClicked(button->text());
    }
}

void ThemedFormWidget::applyTheme() {
    setStyleSheet(Theme::instance().getInputStyle());
}