#include "Config.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QStandardPaths>
#include <QTextStream>

// 返回全局 Config 实例
Config& Config::instance() {
    static Config s;
    return s;
}

// 计算配置文件所在目录：优先使用平台建议目录
QString Config::configDir() const {
    const QString base = QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation);
    if (!base.isEmpty()) return base;
    // 兜底：使用 ~./config/SQLER
    return QDir::homePath() + "/.config/SQLER";
}

// 配置文件完整路径（config.toml）
QString Config::configFile() const { return QDir(configDir()).filePath("config.toml"); }

// 极简 TOML 行解析：仅支持 key = "value" 或 key = value 的简单形式
bool Config::parseTomlLine(const QString& line, QString& key, QString& value) {
    QString s = line.trimmed();
    if (s.isEmpty() || s.startsWith('#')) return false; // 注释或空行
    int eq = s.indexOf('=');
    if (eq <= 0) return false; // 无等号或 key 为空
    key = s.left(eq).trimmed();
    QString rhs = s.mid(eq + 1).trimmed();
    // 支持双引号包裹的字符串
    if (rhs.startsWith('"') && rhs.endsWith('"') && rhs.size() >= 2) {
        value = rhs.mid(1, rhs.size() - 2);
    } else {
        value = rhs; // 简单未加引号的值（不做转义处理）
    }
    return true;
}

// 将字符串中的引号/反斜杠转义，以便写入 TOML 字符串
QString Config::escapeTomlString(const QString& s) {
    QString out;
    out.reserve(s.size());
    for (QChar c : s) {
        if (c == '\\' || c == '"') out.append('\\');
        out.append(c);
    }
    return out;
}

// 从磁盘加载配置（若文件不存在则创建目录并返回 true）
bool Config::load() {
    const QString filePath = configFile();
    QFile f(filePath);
    if (!f.exists()) {
        QDir().mkpath(QFileInfo(filePath).dir().absolutePath());
        // 首次运行：不强制生成文件，使用默认值
        return true;
    }
    if (!f.open(QIODevice::ReadOnly | QIODevice::Text)) return false;
    QTextStream ts(&f);
    while (!ts.atEnd()) {
        const QString line = ts.readLine();
        QString k, v;
        if (!parseTomlLine(line, k, v)) continue;
        if (k == QLatin1String("data_dir")) m_cfg.dataDir = v;
    }
    return true;
}

// 将当前配置写入到 config.toml
bool Config::save() const {
    const QString filePath = configFile();
    QDir().mkpath(QFileInfo(filePath).dir().absolutePath());
    QFile f(filePath);
    if (!f.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) return false;
    QTextStream ts(&f);
    ts << "# SQLER config\n";
    if (!m_cfg.dataDir.isEmpty()) {
        ts << "data_dir = \"" << escapeTomlString(m_cfg.dataDir) << "\"\n";
    } else {
        ts << "# data_dir = \"/path/to/appdata\"\n";
    }
    return true;
}

// 设置数据目录（仅修改内存，需调用 save() 持久化）
void Config::setDataDir(QString dir) { m_cfg.dataDir = std::move(dir); }
