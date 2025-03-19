#include "chatmessage.h"
#include <QStringList>
#include <QTextDocument>

ChatMessage::ChatMessage()
    : m_color(QColor("#FFFFFF"))
    , m_timestamp(QDateTime::currentDateTime())
{
}

ChatMessage::ChatMessage(const QString &username, const QString &message, const QColor &color)
    : m_username(username)
    , m_message(message)
    , m_color(color)
    , m_timestamp(QDateTime::currentDateTime())
{
}

QString ChatMessage::username() const
{
    return m_username;
}

void ChatMessage::setUsername(const QString &username)
{
    m_username = username;
}

QString ChatMessage::message() const
{
    return m_message;
}

void ChatMessage::setMessage(const QString &message)
{
    m_message = message;
}

QColor ChatMessage::color() const
{
    return m_color;
}

void ChatMessage::setColor(const QColor &color)
{
    m_color = color;
}

QDateTime ChatMessage::timestamp() const
{
    return m_timestamp;
}

void ChatMessage::setTimestamp(const QDateTime &timestamp)
{
    m_timestamp = timestamp;
}

QMap<QString, QString> ChatMessage::badges() const
{
    return m_badges;
}

void ChatMessage::setBadges(const QMap<QString, QString> &badges)
{
    m_badges = badges;
}

void ChatMessage::addBadge(const QString &badge, const QString &version)
{
    m_badges.insert(badge, version);
}

QMap<QString, QString> ChatMessage::tags() const
{
    return m_tags;
}

void ChatMessage::setTags(const QMap<QString, QString> &tags)
{
    m_tags = tags;
}

void ChatMessage::addTag(const QString &key, const QString &value)
{
    m_tags.insert(key, value);
}

QString ChatMessage::formattedMessage(int fontSize) const
{
    // Échapper le HTML pour empêcher l'injection
    QTextDocument doc;
    doc.setHtml(m_message);
    QString escapedMessage = doc.toPlainText();
    
    // Formatter avec des couleurs
    QString formatted = QString("<div style='margin-bottom: 4px;'>");
    
    // Si on a un badge de modérateur, on l'ajoute
    if (m_badges.contains("moderator")) {
        formatted += QString("<span style='color: #00FF00; font-weight: bold;'>&#9679;</span> ");
    }
    
    // Ajouter le nom d'utilisateur formaté avec taille de police adaptée
    formatted += QString("<span style='color: %1; font-weight: bold; font-size: %2px;'>%3:</span> ")
                   .arg(m_color.name())
                   .arg(fontSize)
                   .arg(m_username);
    
    // Ajouter le message avec taille de police adaptée
    formatted += QString("<span style='font-size: %1px;'>%2</span></div>")
                   .arg(fontSize)
                   .arg(escapedMessage);
    
    return formatted;
} 