#ifndef CHATMESSAGE_H
#define CHATMESSAGE_H

#include <QString>
#include <QColor>
#include <QMap>
#include <QDateTime>

class ChatMessage
{
public:
    ChatMessage();
    ChatMessage(const QString &username, const QString &message, const QColor &color = QColor("#FFFFFF"));

    QString username() const;
    void setUsername(const QString &username);

    QString message() const;
    void setMessage(const QString &message);

    QColor color() const;
    void setColor(const QColor &color);

    QDateTime timestamp() const;
    void setTimestamp(const QDateTime &timestamp);

    // Badges et tags Twitch
    QMap<QString, QString> badges() const;
    void setBadges(const QMap<QString, QString> &badges);
    void addBadge(const QString &badge, const QString &version);

    QMap<QString, QString> tags() const;
    void setTags(const QMap<QString, QString> &tags);
    void addTag(const QString &key, const QString &value);

    // Formatage pour l'affichage
    QString formattedMessage(int fontSize = 14) const;

private:
    QString m_username;
    QString m_message;
    QColor m_color;
    QDateTime m_timestamp;
    QMap<QString, QString> m_badges;
    QMap<QString, QString> m_tags;
};

#endif // CHATMESSAGE_H 