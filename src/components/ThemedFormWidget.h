#pragma once

#include <QWidget>

class QFormLayout;
class ThemedButton;

class ThemedFormWidget : public QWidget {
    Q_OBJECT

public:
    explicit ThemedFormWidget(QWidget* parent = nullptr);

    void addFormRow(const QString& label, QWidget* field);
    void addFormButtons(const QStringList& leftButtons = {}, const QStringList& rightButtons = {});

    [[nodiscard]] ThemedButton* getButton(const QString& text) const;

signals:
    void buttonClicked(const QString& buttonText);

protected:
    void setupForm();

private slots:
    void onThemeChanged();
    void onButtonClicked();

private:
    void applyTheme();

    QFormLayout* m_formLayout;
    QWidget* m_buttonContainer;
    QList<ThemedButton*> m_buttons;
};