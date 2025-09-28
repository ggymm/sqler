#include "MainWindow.h"
#include "components/GTheme.h"

#include <QApplication>

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);

    // Apply light theme globally; switch to Dark via setMode if needed
    GTheme::instance().setMode(GTheme::Mode::Light);
    GTheme::instance().applyToApp();

    MainWindow window;
    window.show();

    return QApplication::exec();
}
