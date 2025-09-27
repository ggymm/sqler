#include "MainWindow.h"
#include "TopMenuBar.h"
#include "ConnectionPanel.h"
#include "MainContent.h"
#include "Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QWidget>
#include <QFrame>

MainWindow::MainWindow(QWidget* parent)
    : QMainWindow(parent)
    , m_topMenuBar(nullptr)
    , m_connectionPanel(nullptr)
    , m_mainContent(nullptr) {
    setupUI();
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &MainWindow::onThemeChanged);
}

void MainWindow::setupUI() {
    setWindowTitle(QStringLiteral("SQL Database Manager"));
    resize(1200, 800);

    auto* centralWidget = new QWidget(this);
    setCentralWidget(centralWidget);

    auto* mainLayout = new QVBoxLayout(centralWidget);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    m_topMenuBar = new TopMenuBar(this);
    m_topMenuBar->setFixedHeight(Theme::Sizes::topBarHeight);
    mainLayout->addWidget(m_topMenuBar);

    auto* contentLayout = new QHBoxLayout();
    contentLayout->setContentsMargins(0, 0, 0, 0);
    contentLayout->setSpacing(0);

    m_connectionPanel = new ConnectionPanel(this);
    m_connectionPanel->setFixedWidth(Theme::Sizes::sideBarWidth);
    contentLayout->addWidget(m_connectionPanel);

    auto* separator = new QFrame(this);
    separator->setFrameShape(QFrame::VLine);
    separator->setLineWidth(1);
    separator->setFixedWidth(1);
    contentLayout->addWidget(separator);

    m_mainContent = new MainContent(this);
    contentLayout->addWidget(m_mainContent);

    auto* contentWidget = new QWidget(this);
    contentWidget->setLayout(contentLayout);
    mainLayout->addWidget(contentWidget);
}

void MainWindow::applyTheme() {
    const auto& colors = Theme::instance().colors();

    const QString styleSheet = QStringLiteral(
        "QMainWindow { background-color: %1; }"
        "QFrame[frameShape=\"5\"] { color: %2; }"  // VLine separator
    ).arg(colors.background.name(), colors.border.name());

    setStyleSheet(styleSheet);
}

void MainWindow::onThemeChanged() {
    applyTheme();
}