#ifndef IRCCLIENT_H
#define IRCCLIENT_H

#include <QObject>
#include <QTcpSocket>
#include <QTimer>
#include <QStringList>
#include "chatmessage.h"

class IrcClient : public QObject
{
    Q_OBJECT
public:
    explicit IrcClient(QObject *parent = nullptr);
    ~IrcClient();

    void connectToServer(const QString &channel);
    void disconnectFromServer();
    bool isConnected() const;
    void sendMessage(const QString &message);
    void setAutoReconnect(bool enable, int intervalMs = 10000);

signals:
    void connected();
    void disconnected();
    void messageReceived(const ChatMessage &message);
    void errorOccurred(const QString &errorMessage);
    void reconnecting(int attempt);

private slots:
    void onConnected();
    void onDisconnected();
    void onReadyRead();
    void onError(QAbstractSocket::SocketError socketError);
    void onPingSent();
    void attemptReconnect();

private:
    void parseMessage(const QString &message);
    void sendPong(const QString &server);
    QString generateRandomNickname() const;

    QTcpSocket *socket;
    QString nickname;
    QString channelName;
    QTimer *pingTimer;
    QTimer *reconnectTimer;
    bool authenticated;
    bool autoReconnect;
    int reconnectInterval;
    int reconnectAttempts;
    int maxReconnectAttempts;

    // Constantes pour la connexion IRC Twitch
    const QString SERVER = "irc.chat.twitch.tv";
    const int PORT = 6667;
    // Le token doit être au format "oauth:xxxxxx" mais pour les utilisateurs anonymes on peut utiliser SCHMOOPIIE
    const QString DEFAULT_TOKEN = "SCHMOOPIIE";
};

#endif // IRCCLIENT_H 