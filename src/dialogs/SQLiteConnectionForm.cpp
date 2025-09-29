#include "SQLiteConnectionForm.h"

#include "../components/GLabel.h"
#include "../components/GLineEdit.h"
#include "../components/GPushButton.h"

#include <QFileDialog>
#include <QFormLayout>

SQLiteConnectionForm::SQLiteConnectionForm(QWidget* parent) : ConnectionFormBase(parent) { setupUI(); }

void SQLiteConnectionForm::setupUI() {
    // Connection Name
    m_nameEdit = new GLineEdit(this);
    m_nameEdit->setText("SQLite 数据库");
    m_nameEdit->setPlaceholderText("我的SQLite数据库");
    m_formLayout->addRow(new GLabel("连接名称:"), m_nameEdit);

    // File Path with buttons
    auto* fileWidget = new QWidget(this);
    auto* fileLayout = new QHBoxLayout(fileWidget);
    fileLayout->setContentsMargins(0, 0, 0, 0);
    fileLayout->setSpacing(8);

    m_filePathEdit = new GLineEdit(fileWidget);
    m_filePathEdit->setPlaceholderText("选择或输入数据库文件路径");
    m_filePathEdit->setReadOnly(true);
    fileLayout->addWidget(m_filePathEdit);

    m_browseButton = new GPushButton("浏览", GPushButton::Variant::Secondary, fileWidget);
    m_browseButton->setFixedWidth(80);
    m_browseButton->setObjectName("browseButton");
    connect(m_browseButton, &QPushButton::clicked, this, &SQLiteConnectionForm::browseFile);
    fileLayout->addWidget(m_browseButton);

    m_newFileButton = new GPushButton("新建", GPushButton::Variant::Secondary, fileWidget);
    m_newFileButton->setFixedWidth(80);
    m_newFileButton->setObjectName("newFileButton");
    connect(m_newFileButton, &QPushButton::clicked, this, &SQLiteConnectionForm::createNewFile);
    fileLayout->addWidget(m_newFileButton);

    m_formLayout->addRow(new GLabel("数据库文件:"), fileWidget);
}

void SQLiteConnectionForm::browseFile() {
    QString fileName = QFileDialog::getOpenFileName(this, "选择SQLite数据库文件", QString(), "SQLite数据库 (*.db *.sqlite *.sqlite3);;所有文件 (*)");

    if (!fileName.isEmpty()) {
        m_filePathEdit->setText(fileName);
    }
}

void SQLiteConnectionForm::createNewFile() {
    QString fileName = QFileDialog::getSaveFileName(this, "新建SQLite数据库文件", QString(), "SQLite数据库 (*.db *.sqlite *.sqlite3);;所有文件 (*)");

    if (!fileName.isEmpty()) {
        m_filePathEdit->setText(fileName);
    }
}

QVariantMap SQLiteConnectionForm::getConnectionData() const {
    QVariantMap data;
    data["type"] = "sqlite";
    data["name"] = m_nameEdit->text();
    data["filePath"] = m_filePathEdit->text();
    return data;
}

bool SQLiteConnectionForm::validateInput() const { return !m_nameEdit->text().isEmpty() && !m_filePathEdit->text().isEmpty(); }
