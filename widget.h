#ifndef WIDGET_H
#define WIDGET_H

#include <QWidget>
#include <QSystemTrayIcon>
#include <QMenu>
#include <QAction>
#include <QSettings>
#include <QTimer>
#include <QTextBrowser>
#include <QScrollBar>
#include <QMouseEvent>
#include <QProcess>
#include "ircclient.h"
#include "chatmessage.h"

class Widget : public QWidget
{
    Q_OBJECT

public:
    Widget(QWidget *parent = nullptr);
    ~Widget();

protected:
    void mousePressEvent(QMouseEvent *event) override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;

private slots:
    void showOptionsDialog();
    void showAboutDialog();
    void trayIconActivated(QSystemTrayIcon::ActivationReason reason);
    void onMessageReceived(const ChatMessage &message);
    void onIrcConnected();
    void onIrcDisconnected();
    void onIrcError(const QString &errorMessage);
    void onIrcReconnecting(int attempt);
    void connectToTwitchChat();
    void resetSettings();

private:
    void createTrayIcon();
    void setupOverlayWindow();
    void loadSettings();
    void saveSettings();
    void appendChatMessage(const ChatMessage &message);
    QString getConfigFilePath() const;
    bool checkAndCreateConfigFile();
    void setTransparentForMouseEvents(bool transparent);
    void makeAllChildrenTransparent(QWidget* widget, bool transparent);

    QSystemTrayIcon *trayIcon;
    QMenu *trayIconMenu;
    QAction *optionsAction;
    QAction *aboutAction;
    QAction *quitAction;

    // Settings
    QString twitchChannel;
    int fontSize;
    int bgOpacity;
    int chatWidth;
    int chatHeight;
    int maxMessages;
    bool autoReconnect;
    int reconnectInterval;

    // Chat display
    QTextBrowser *chatDisplay;
    IrcClient *ircClient;
    QList<ChatMessage> messageHistory;
    
    // Interface
    QWidget *backgroundWidget;
    
    // Pour le déplacement
    QPoint dragPosition;
    bool isDragging;
};
#endif // WIDGET_H
