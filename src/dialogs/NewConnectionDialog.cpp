#include "NewConnectionDialog.h"

#include "../components/GLabel.h"
#include "../components/GPushButton.h"
#include "../components/GSeparator.h"
#include "../components/GStyle.h"
#include "ConnectionFormBase.h"
#include "DatabaseTypeDialog.h"
#include "MongoDBConnectionForm.h"
#include "MySQLConnectionForm.h"
#include "OracleConnectionForm.h"
#include "PostgreSQLConnectionForm.h"
#include "RedisConnectionForm.h"
#include "SQLServerConnectionForm.h"
#include "SQLiteConnectionForm.h"

#include <QStackedWidget>
#include <QVBoxLayout>

NewConnectionDialog::NewConnectionDialog(QWidget* parent) : GDialog(parent), m_stackedWidget(nullptr), m_typeDialog(nullptr), m_currentForm(nullptr) {
    setupUI();
}

void NewConnectionDialog::setupUI() {
    setWindowTitle("新建数据库连接");
    resize(600, 500);

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    // Header container so we can style title color specifically
    auto* headerWidget = new QWidget(this);
    headerWidget->setObjectName("dialogHeader");
    auto* headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::md);
    headerLayout->setSpacing(GStyle::Spacing::md);

    m_backButton = new GPushButton("← 返回", GPushButton::Variant::Secondary, this);
    m_backButton->setVisible(false);
    connect(m_backButton, &QPushButton::clicked, this, &NewConnectionDialog::onBackClicked);
    headerLayout->addWidget(m_backButton);

    headerLayout->addStretch();

    auto* titleLabel = new GLabel("新建数据库连接", GLabel::Role::Emphasis, this);
    headerLayout->addWidget(titleLabel);

    headerLayout->addStretch();

    layout->addWidget(headerWidget);

    auto* separator = new GSeparator(GSeparator::Orientation::Horizontal, this);
    layout->addWidget(separator);

    m_stackedWidget = new QStackedWidget(this);
    m_stackedWidget->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    layout->addWidget(m_stackedWidget, 1); // Give it stretch factor so it expands

    // Footer container so we can show/hide the whole footer without leaving blank space
    m_footerWidget = new QWidget(this);
    m_footerWidget->setObjectName("dialogFooter");
    m_buttonLayout = new QHBoxLayout(m_footerWidget);
    m_buttonLayout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::md, GStyle::Spacing::lg, GStyle::Spacing::lg);
    m_buttonLayout->addStretch();

    m_cancelButton = new GPushButton("取消", GPushButton::Variant::Secondary, this);
    connect(m_cancelButton, &QPushButton::clicked, this, &QDialog::reject);
    m_buttonLayout->addWidget(m_cancelButton);

    layout->addWidget(m_footerWidget);

    showDatabaseTypeSelection();
}

void NewConnectionDialog::showDatabaseTypeSelection() {
    if (m_typeDialog) {
        m_stackedWidget->removeWidget(m_typeDialog);
        m_typeDialog->deleteLater();
    }

    m_typeDialog = new DatabaseTypeDialog(this);
    connect(m_typeDialog, &DatabaseTypeDialog::selected, this, &NewConnectionDialog::showConnectionForm);

    m_stackedWidget->addWidget(m_typeDialog);
    m_stackedWidget->setCurrentWidget(m_typeDialog);
    m_backButton->setVisible(false);

    // Reset footer to show only Cancel for database type selection
    while (QLayoutItem* item = m_buttonLayout->takeAt(0)) {
        if (auto* w = item->widget()) {
            w->setParent(nullptr);
        }
        delete item;
    }
    m_buttonLayout->addStretch();
    m_buttonLayout->addWidget(m_cancelButton);
    m_footerWidget->setVisible(true);
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

        // Populate dialog footer with form's footer buttons (full-width, bottom-anchored)
        m_currentForm->attachFooterTo(m_buttonLayout);
        m_footerWidget->setVisible(true);
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
    } else if (databaseType == "oracle") {
        return new OracleConnectionForm(this);
    } else if (databaseType == "sqlserver") {
        return new SQLServerConnectionForm(this);
    } else {
        return nullptr;
    }
}

// No page-level styles

void NewConnectionDialog::onBackClicked() { showDatabaseTypeSelection(); }

void NewConnectionDialog::onConnectionSaved() { accept(); }
