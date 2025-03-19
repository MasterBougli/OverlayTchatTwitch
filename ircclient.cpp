#include "ircclient.h"
#include <QRegularExpression>
#include <QDebug>
#include <QRandomGenerator>

IrcClient::IrcClient(QObject *parent)
    : QObject(parent)
    , socket(new QTcpSocket(this))
    , nickname(generateRandomNickname())
    , pingTimer(new QTimer(this))
    , reconnectTimer(new QTimer(this))
    , authenticated(false)
    , autoReconnect(false)
    , reconnectInterval(10000) // 10 secondes par défaut
    , reconnectAttempts(0)
    , maxReconnectAttempts(5)
{
    connect(socket, &QTcpSocket::connected, this, &IrcClient::onConnected);
    connect(socket, &QTcpSocket::disconnected, this, &IrcClient::onDisconnected);
    connect(socket, &QTcpSocket::readyRead, this, &IrcClient::onReadyRead);
    
    // Nouvelle syntaxe pour Qt 6
    connect(socket, static_cast<void(QTcpSocket::*)(QAbstractSocket::SocketError)>(&QTcpSocket::errorOccurred), 
            this, &IrcClient::onError);
    
    // Configurer le timer pour envoyer des PING périodiques
    connect(pingTimer, &QTimer::timeout, this, &IrcClient::onPingSent);
    pingTimer->setInterval(60000); // Une minute
    
    // Configurer le timer de reconnexion
    connect(reconnectTimer, &QTimer::timeout, this, &IrcClient::attemptReconnect);
    reconnectTimer->setSingleShot(true);
}

IrcClient::~IrcClient()
{
    disconnectFromServer();
    delete socket;
    delete pingTimer;
    delete reconnectTimer;
}

void IrcClient::connectToServer(const QString &channel)
{
    if (socket->state() == QAbstractSocket::ConnectedState) {
        disconnectFromServer();
    }
    
    channelName = channel.toLower();
    if (!channelName.startsWith("#")) {
        channelName = "#" + channelName;
    }
    
    // Réinitialiser les tentatives de reconnexion
    reconnectAttempts = 0;
    
    // Générer un nouveau nom d'utilisateur aléatoire à chaque connexion
    nickname = generateRandomNickname();
    
    qDebug() << "Connexion au chat Twitch pour le canal:" << channelName << "avec le nickname:" << nickname;
    socket->connectToHost(SERVER, PORT);
}

void IrcClient::disconnectFromServer()
{
    reconnectTimer->stop();
    
    if (socket->state() == QAbstractSocket::ConnectedState) {
        socket->disconnectFromHost();
    }
    
    pingTimer->stop();
    authenticated = false;
}

void IrcClient::setAutoReconnect(bool enable, int intervalMs)
{
    autoReconnect = enable;
    reconnectInterval = intervalMs;
    qDebug() << "Reconnexion automatique" << (enable ? "activée" : "désactivée") 
             << "avec intervalle de" << intervalMs << "ms";
}

void IrcClient::attemptReconnect()
{
    if (!autoReconnect || reconnectAttempts >= maxReconnectAttempts) {
        qDebug() << "Nombre maximal de tentatives de reconnexion atteint (" << reconnectAttempts << ")";
        reconnectTimer->stop();
        return;
    }
    
    reconnectAttempts++;
    emit reconnecting(reconnectAttempts);
    
    qDebug() << "Tentative de reconnexion" << reconnectAttempts << "sur" << maxReconnectAttempts;
    
    // Se reconnecter avec un nouveau nickname
    nickname = generateRandomNickname();
    socket->connectToHost(SERVER, PORT);
}

bool IrcClient::isConnected() const
{
    return socket->state() == QAbstractSocket::ConnectedState && authenticated;
}

void IrcClient::sendMessage(const QString &message)
{
    if (isConnected()) {
        QString cmd = QString("PRIVMSG %1 :%2\r\n").arg(channelName).arg(message);
        socket->write(cmd.toUtf8());
    }
}

void IrcClient::onConnected()
{
    qDebug() << "Connecté au serveur IRC de Twitch";
    
    // Authentification - pour les connexions anonymes, on utilise le format justinfanXXXXX
    socket->write(QString("PASS %1\r\n").arg(DEFAULT_TOKEN).toUtf8());
    socket->write(QString("NICK %1\r\n").arg(nickname).toUtf8());
    socket->write(QString("USER %1 8 * :%1\r\n").arg(nickname).toUtf8());
    
    // Demander des tags Twitch
    socket->write("CAP REQ :twitch.tv/tags\r\n");
    socket->write("CAP REQ :twitch.tv/commands\r\n");
    socket->write("CAP REQ :twitch.tv/membership\r\n");
    
    // Rejoindre le canal
    socket->write(QString("JOIN %1\r\n").arg(channelName).toUtf8());
    
    // Démarrer le timer de ping
    pingTimer->start();
    
    authenticated = true;
    emit connected();
}

void IrcClient::onDisconnected()
{
    qDebug() << "Déconnecté du serveur IRC de Twitch";
    pingTimer->stop();
    authenticated = false;
    emit disconnected();
    
    // Lancer la reconnexion automatique si activée
    if (autoReconnect && reconnectAttempts < maxReconnectAttempts) {
        qDebug() << "Planification d'une reconnexion dans" << reconnectInterval << "ms";
        reconnectTimer->start(reconnectInterval);
    }
}

void IrcClient::onReadyRead()
{
    while (socket->canReadLine()) {
        QString line = QString::fromUtf8(socket->readLine()).trimmed();
        qDebug() << "IRC <<<" << line;
        
        // Traiter les PING du serveur
        if (line.startsWith("PING")) {
            QString server = line.mid(5);
            sendPong(server);
            continue;
        }
        
        // Détecter les erreurs de connexion
        if (line.contains("NOTICE") && (line.contains("Login authentication failed") || 
                                        line.contains("Invalid NICK") || 
                                        line.contains("Improperly formatted auth"))) {
            qDebug() << "Erreur d'authentification IRC:" << line;
            emit errorOccurred(tr("Erreur d'authentification: ") + line);
            disconnectFromServer();
            return;
        }
        
        parseMessage(line);
    }
}

void IrcClient::onError(QAbstractSocket::SocketError socketError)
{
    QString errorMessage;
    switch (socketError) {
        case QAbstractSocket::RemoteHostClosedError:
            errorMessage = tr("Le serveur a fermé la connexion");
            break;
        case QAbstractSocket::HostNotFoundError:
            errorMessage = tr("Serveur introuvable");
            break;
        case QAbstractSocket::ConnectionRefusedError:
            errorMessage = tr("Connexion refusée par le serveur");
            break;
        case QAbstractSocket::SocketTimeoutError:
            errorMessage = tr("Délai d'attente dépassé pour la connexion");
            break;
        case QAbstractSocket::NetworkError:
            errorMessage = tr("Erreur réseau");
            break;
        default:
            errorMessage = tr("Erreur: ") + socket->errorString();
    }
    
    qDebug() << "Erreur de socket:" << errorMessage;
    emit errorOccurred(errorMessage);
}

void IrcClient::onPingSent()
{
    if (isConnected()) {
        socket->write("PING :tmi.twitch.tv\r\n");
    }
}

void IrcClient::parseMessage(const QString &message)
{
    // Regex pour les messages Twitch en format IRCv3 avec tags
    QRegularExpression msgRegex(
        "^(?:@([^ ]+) )?(?::([^ ]+) )?([^ ]+)(?: ([^:][^ ]*(?: [^:][^ ]*)*))?(?: :(.*))?$"
    );
    
    QRegularExpressionMatch match = msgRegex.match(message);
    
    if (match.hasMatch()) {
        QString tags = match.captured(1);
        QString prefix = match.captured(2);
        QString command = match.captured(3);
        QString params = match.captured(4);
        QString trailing = match.captured(5);
        
        // Ignorer les commandes non PRIVMSG (pas des messages de chat)
        if (command != "PRIVMSG") {
            return;
        }
        
        // Extraire les informations de l'utilisateur
        QString nick;
        if (prefix.contains("!")) {
            nick = prefix.left(prefix.indexOf("!"));
        } else {
            nick = prefix;
        }
        
        // Extraire le message
        QString chatMessage = trailing;
        
        // Traiter les tags IRCv3
        QMap<QString, QString> tagMap;
        if (!tags.isEmpty()) {
            QStringList tagList = tags.split(';');
            for (const QString &tag : tagList) {
                QStringList keyValue = tag.split('=');
                if (keyValue.size() == 2) {
                    tagMap[keyValue[0]] = keyValue[1];
                }
            }
        }
        
        // Créer un objet message
        ChatMessage chatMsg(nick, chatMessage);
        
        // Ajouter les couleurs et les badges
        if (tagMap.contains("color") && !tagMap["color"].isEmpty()) {
            chatMsg.setColor(QColor(tagMap["color"]));
        }
        
        // Ajouter les badges
        if (tagMap.contains("badges") && !tagMap["badges"].isEmpty()) {
            QStringList badges = tagMap["badges"].split(',');
            for (const QString &badge : badges) {
                QStringList badgeParts = badge.split('/');
                if (badgeParts.size() == 2) {
                    chatMsg.addBadge(badgeParts[0], badgeParts[1]);
                }
            }
        }
        
        // Ajouter tous les tags au message
        chatMsg.setTags(tagMap);
        
        // Envoyer le message
        emit messageReceived(chatMsg);
    }
}

void IrcClient::sendPong(const QString &server)
{
    socket->write(QString("PONG %1\r\n").arg(server).toUtf8());
}

QString IrcClient::generateRandomNickname() const 
{
    // Générer un nom d'utilisateur au format justinfanXXXXX où X est un chiffre aléatoire
    QString base = "justinfan";
    for (int i = 0; i < 5; i++) {
        base += QString::number(QRandomGenerator::global()->bounded(10));
    }
    return base;
} 