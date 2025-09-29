#include "MainWindow.h"

#include "ConnectionPanel.h"
#include "MainContent.h"
#include "TopMenuBar.h"
#include "components/GSeparator.h"
#include "components/GStyle.h"
#include "components/GTheme.h"

#include <QHBoxLayout>

MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent), m_topMenuBar(nullptr), m_connectionPanel(nullptr), m_mainContent(nullptr)
{
    setupUI();
}

void MainWindow::setupUI()
{
    setWindowTitle(QStringLiteral("SQL Database Manager"));
    resize(1200, 800);

    auto* centralWidget = new QWidget(this);
    setCentralWidget(centralWidget);

    auto* mainLayout = new QVBoxLayout(centralWidget);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    m_topMenuBar = new TopMenuBar(this);
    m_topMenuBar->setFixedHeight(GStyle::Sizes::topBarHeight);
    mainLayout->addWidget(m_topMenuBar);

    // Connect theme toggle signal
    connect(m_topMenuBar, &TopMenuBar::themeToggleClicked, this,
            [this]()
            {
                auto& theme = GTheme::instance();
                const auto newMode = theme.mode() == GTheme::Mode::Light ? GTheme::Mode::Dark : GTheme::Mode::Light;
                theme.setMode(newMode);
            });

    auto* contentLayout = new QHBoxLayout();
    contentLayout->setContentsMargins(0, 0, 0, 0);
    contentLayout->setSpacing(0);

    m_connectionPanel = new ConnectionPanel(this);
    m_connectionPanel->setFixedWidth(GStyle::Sizes::sideBarWidth);
    contentLayout->addWidget(m_connectionPanel);

    auto* separator = new GSeparator(GSeparator::Orientation::Vertical, this);
    contentLayout->addWidget(separator);

    m_mainContent = new MainContent(this);
    contentLayout->addWidget(m_mainContent);

    auto* contentWidget = new QWidget(this);
    contentWidget->setLayout(contentLayout);
    mainLayout->addWidget(contentWidget);
}

// No page-level styles
