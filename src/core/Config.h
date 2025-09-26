#pragma once

#include <QString>

// 应用配置数据结构
// - dataDir: 应用数据目录（例如保存 connections.json 的位置）
//   若为空，使用平台默认的 AppData 位置
struct AppConfig {
    QString dataDir; // 应用数据目录（空表示默认位置）
};

// 配置管理单例
// 负责从 TOML 文件加载/保存配置项，并提供读取/修改接口
class Config final {
public:
    // 获取全局唯一实例
    static Config& instance();

    // 从磁盘加载配置（若文件不存在则保持默认值）
    bool load();
    // 保存当前配置到磁盘
    bool save() const;

    // 读取当前配置结构
    const AppConfig& get() const { return m_cfg; }
    // 设置数据目录（调用后可 save() 持久化）
    void setDataDir(QString dir);

    // 配置文件所在目录与文件路径（平台相关）
    QString configDir() const;
    QString configFile() const;

private:
    Config() = default;

    // 解析 TOML 的简单行：形如 key = "value"
    static bool parseTomlLine(const QString& line, QString& key, QString& value);
    // 将字符串按 TOML 规则进行转义（仅转义引号与反斜杠）
    static QString escapeTomlString(const QString& s);

private:
    AppConfig m_cfg; // 内存中的配置副本
};
