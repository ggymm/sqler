#pragma once

#include <QWidget>

class GLabel;
class GPushButton;

class MainContent final : public QWidget {
    Q_OBJECT

  public:
    explicit MainContent(QWidget* parent = nullptr);

  private:
    void setupUI();
    GPushButton* createQuickActionButton(const QString& title, const QString& description, const QString& iconPath);

    GLabel* m_titleLabel;
    GLabel* m_subtitleLabel;
    GPushButton* m_newConnectionAction;
    GPushButton* m_newQueryAction;
    GPushButton* m_browseTablesAction;
};
