#include "NewConnectionDialog.h"
#include "DatabaseTypeDialog.h"
#include "ConnectionFormBase.h"
#include "MySQLConnectionForm.h"
#include "PostgreSQLConnectionForm.h"
#include "SQLiteConnectionForm.h"
#include "MongoDBConnectionForm.h"
#include "RedisConnectionForm.h"
#include "../Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QStackedWidget>
#include <QPushButton>
#include <QLabel>
#include <QFrame>

NewConnectionDialog::NewConnectionDialog(QWidget* parent)
    : QDialog(parent)
    , m_stackedWidget(nullptr)
    , m_typeDialog(nullptr)
    , m_currentForm(nullptr) {
    setupUI();
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &NewConnectionDialog::onThemeChanged);
}

void NewConnectionDialog::setupUI() {
    setWindowTitle("新建数据库连接");
    setModal(true);
    resize(600, 500);

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    auto* headerLayout = new QHBoxLayout();
    headerLayout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::md);
    headerLayout->setSpacing(Theme::Spacing::md);

    m_backButton = new QPushButton("← 返回", this);
    m_backButton->setObjectName("backButton");
    m_backButton->setVisible(false);
    connect(m_backButton, &QPushButton::clicked, this, &NewConnectionDialog::onBackClicked);
    headerLayout->addWidget(m_backButton);

    headerLayout->addStretch();

    auto* titleLabel = new QLabel("新建数据库连接", this);
    titleLabel->setObjectName("titleLabel");
    headerLayout->addWidget(titleLabel);

    headerLayout->addStretch();

    layout->addLayout(headerLayout);

    auto* separator = new QFrame(this);
    separator->setFrameShape(QFrame::HLine);
    separator->setLineWidth(1);
    separator->setObjectName("separator");
    layout->addWidget(separator);

    m_stackedWidget = new QStackedWidget(this);
    layout->addWidget(m_stackedWidget);

    m_buttonLayout = new QHBoxLayout();
    m_buttonLayout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::md, Theme::Spacing::lg, Theme::Spacing::lg);
    m_buttonLayout->addStretch();

    m_cancelButton = new QPushButton("取消", this);
    m_cancelButton->setObjectName("cancelButton");
    connect(m_cancelButton, &QPushButton::clicked, this, &QDialog::reject);
    m_buttonLayout->addWidget(m_cancelButton);

    layout->addLayout(m_buttonLayout);

    showDatabaseTypeSelection();
}

void NewConnectionDialog::showDatabaseTypeSelection() {
    if (m_typeDialog) {
        m_stackedWidget->removeWidget(m_typeDialog);
        m_typeDialog->deleteLater();
    }

    m_typeDialog = new DatabaseTypeDialog(this);
    connect(m_typeDialog, &QDialog::accepted, [this]() {
        showConnectionForm(m_typeDialog->selectedDatabaseType());
    });
    connect(m_typeDialog, &QDialog::rejected, this, &QDialog::reject);

    m_stackedWidget->addWidget(m_typeDialog);
    m_stackedWidget->setCurrentWidget(m_typeDialog);
    m_backButton->setVisible(false);

    // Show bottom button layout for database type selection
    for (int i = 0; i < m_buttonLayout->count(); ++i) {
        if (auto* item = m_buttonLayout->itemAt(i)) {
            if (auto* widget = item->widget()) {
                widget->setVisible(true);
            }
        }
    }
}

void NewConnectionDialog::showConnectionForm(const QString& databaseType) {
    if (m_currentForm) {
        m_stackedWidget->removeWidget(m_currentForm);
        m_currentForm->deleteLater();
    }

    m_currentDatabaseType = databaseType;
    m_currentForm = createConnectionForm(databaseType);

    if (m_currentForm) {
        connect(m_currentForm, &ConnectionFormBase::connectionSaved, this, &NewConnectionDialog::onConnectionSaved);
        connect(m_currentForm, &ConnectionFormBase::backClicked, this, &NewConnectionDialog::onBackClicked);
        connect(m_currentForm, &ConnectionFormBase::cancelClicked, this, &QDialog::reject);
        m_stackedWidget->addWidget(m_currentForm);
        m_stackedWidget->setCurrentWidget(m_currentForm);
        m_backButton->setVisible(false); // Hide header back button since form has its own

        // Hide bottom button layout when showing connection form
        for (int i = 0; i < m_buttonLayout->count(); ++i) {
            if (auto* item = m_buttonLayout->itemAt(i)) {
                if (auto* widget = item->widget()) {
                    widget->setVisible(false);
                }
            }
        }
    }
}

ConnectionFormBase* NewConnectionDialog::createConnectionForm(const QString& databaseType) {
    if (databaseType == "mysql") {
        return new MySQLConnectionForm(this);
    } else if (databaseType == "postgresql") {
        return new PostgreSQLConnectionForm(this);
    } else if (databaseType == "sqlite") {
        return new SQLiteConnectionForm(this);
    } else if (databaseType == "mongodb") {
        return new MongoDBConnectionForm(this);
    } else if (databaseType == "redis") {
        return new RedisConnectionForm(this);
    } else {
        return nullptr;
    }
}

void NewConnectionDialog::applyTheme() {
    const auto& colors = Theme::instance().colors();

    QString styleSheet = QString(
        "NewConnectionDialog {"
        "    background-color: %1;"
        "}"
        "QLabel#titleLabel {"
        "    color: %2;"
        "    font-size: 16px;"
        "    font-weight: bold;"
        "}"
        "QPushButton#backButton, QPushButton#cancelButton {"
        "    color: %2;"
        "    background-color: transparent;"
        "    border: none;"
        "    padding: 8px 16px;"
        "    border-radius: %3px;"
        "}"
        "QPushButton#backButton:hover, QPushButton#cancelButton:hover {"
        "    background-color: %4;"
        "}"
        "QFrame#separator {"
        "    color: %5;"
        "}"
    ).arg(colors.surface.name()), colors.surface.name(), colors.text.name()), colors.text.name())
     .arg(colors.border.name());

    setStyleSheet(styleSheet);
}

void NewConnectionDialog::onThemeChanged() {
    applyTheme();
}

void NewConnectionDialog::onBackClicked() {
    showDatabaseTypeSelection();
}

void NewConnectionDialog::onConnectionSaved() {
    accept();
}