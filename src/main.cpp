// Qt Quick/QML entry
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>

#include "core/Config.h"
#include "qml/Backend.h"

int main(int argc, char* argv[]) {
    QGuiApplication app(argc, argv);

    // Optional: choose a consistent style
    QQuickStyle::setStyle(QStringLiteral("Fusion"));

    // Load app config early (e.g., data dir)
    Config::instance().load();

    // Backend bridge exposed to QML
    Backend backend;

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("backend", &backend);

    const QUrl url(QStringLiteral("qrc:/qml/Main.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated, &app,
                     [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);
    engine.load(url);

    return app.exec();
}
