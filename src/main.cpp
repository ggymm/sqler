#include <QApplication>
#include <QLabel>

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    QLabel label("Hello, Qt!");
    label.resize(320, 80);
    label.setAlignment(Qt::AlignCenter);
    label.show();

    return QApplication::exec();
}

