#pragma once

#include <QWidget>

class GPushButton;

class TopMenuBar : public QWidget
{
    Q_OBJECT

  public:
    explicit TopMenuBar(QWidget* parent = nullptr);

  signals:
    void newConnectionClicked();
    void newQueryClicked();
    void tablesClicked();
    void queryClicked();
    void functionsClicked();
    void usersClicked();

  private:
    void setupUI();
    GPushButton* createMenuButton(const QString& text, const QString& iconPath);

    GPushButton* m_newConnectionBtn{};
    GPushButton* m_newQueryBtn{};
    GPushButton* m_tablesBtn{};
    GPushButton* m_queryBtn{};
    GPushButton* m_functionsBtn{};
    GPushButton* m_usersBtn{};
};
