#pragma once

#include <QLabel>

class GLabel : public QLabel
{
    Q_OBJECT

  public:
    enum class Role
    {
        Title,
        Subtitle,
        Body,
        Caption,
        Emphasis
    };

    explicit GLabel(QWidget* parent = nullptr);
    explicit GLabel(const QString& text, QWidget* parent = nullptr);
    GLabel(const QString& text, Role role, QWidget* parent = nullptr);

    void setRole(Role role);

  private:
    void setup();
    void applyStyle();

    Role m_role = Role::Body;
};
