use eframe::egui::Color32;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

pub struct Line(pub String, pub Color32, pub String, pub bool);

fn message(raw: &str) -> Option<Line> {
    let (header, body) = raw.split_once(" PRIVMSG ")?;
    let (_, body) = body.split_once(" :")?;
    let name = header
        .split_whitespace()
        .last()?
        .trim_start_matches(':')
        .split('!')
        .next()?
        .to_owned();
    let mut color = Color32::WHITE;
    let mut moderator = false;
    if let Some(tags) = header.strip_prefix('@') {
        for field in tags.split_whitespace().next()?.split(';') {
            if let Some(value) = field
                .strip_prefix("color=#")
                .and_then(|v| u32::from_str_radix(v, 16).ok())
            {
                color = Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8);
            }
            if let Some(value) = field.strip_prefix("badges=") {
                moderator = value.split(',').any(|b| b.starts_with("moderator/"));
            }
        }
    }
    Some(Line(name, color, body.to_string(), moderator))
}

pub fn start(
    channel: String,
    policy: std::sync::Arc<std::sync::Mutex<(bool, u64)>>,
    tx: Sender<Line>,
    stop: std::sync::mpsc::Receiver<()>,
) {
    thread::spawn(move || {
        let status = |s: String| {
            tx.send(Line(String::new(), Color32::WHITE, s, false))
                .is_ok()
        };
        let mut attempts = 0;
        loop {
            let run = || -> Result<(), Box<dyn std::error::Error>> {
                let tcp = TcpStream::connect("irc.chat.twitch.tv:6697")?;
                tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
                tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
                let mut tls =
                    native_tls::TlsConnector::new()?.connect("irc.chat.twitch.tv", tcp)?;
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .subsec_nanos()
                    % 100000;
                let nick = format!("justinfan{nonce:05}");
                let channel = channel.trim_start_matches('#').to_ascii_lowercase();
                tls.write_all(format!("PASS SCHMOOPIIE\r\nNICK {nick}\r\nUSER {nick} 8 * :{nick}\r\nCAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership\r\nJOIN #{channel}\r\n").as_bytes())?;
                let mut ping = std::time::Instant::now();
                let mut reader = BufReader::new(tls);
                let mut raw = String::new();
                while matches!(stop.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)) {
                    if ping.elapsed() >= Duration::from_secs(60) {
                        reader.get_mut().write_all(b"PING :tmi.twitch.tv\r\n")?;
                        ping = std::time::Instant::now();
                    }
                    match reader.read_line(&mut raw) {
                        Ok(0) => break,
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                            ) =>
                        {
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                        Ok(_) => {}
                    }
                    let line = raw.trim_end();
                    if let Some(payload) = line.strip_prefix("PING ") {
                        writeln!(reader.get_mut(), "PONG {payload}\r")?;
                    } else if line.split_whitespace().nth(1) == Some("001") {
                        status(format!("Connecté au chat de {channel}..."));
                    } else if line.contains(" NOTICE ")
                        && [
                            "Login authentication failed",
                            "Invalid NICK",
                            "Improperly formatted auth",
                        ]
                        .iter()
                        .any(|s| line.contains(s))
                    {
                        return Err(format!("Erreur d’authentification : {line}").into());
                    } else if let Some(msg) = message(line) {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                    raw.clear();
                }
                Ok(())
            };
            let result = run();
            let (retry, delay) = *policy.lock().unwrap();
            if !status(
                result
                    .err()
                    .map_or_else(|| "Déconnecté du chat".into(), |e| format!("Erreur : {e}")),
            ) || !retry
                || attempts >= 5
            {
                break;
            }
            if !matches!(
                stop.recv_timeout(Duration::from_millis(delay)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                break;
            }
            if !policy.lock().unwrap().0 {
                break;
            }
            attempts += 1;
            status(format!("Tentative de reconnexion {attempts}..."));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_tags_and_preserves_body() {
        let Line(name,color,body,moderator)=message("@badges=moderator/1;color=#123456;display-name=Bougli :bougli!user PRIVMSG #demo :Salut : tout le monde").unwrap();
        assert_eq!(name, "bougli");
        assert!(moderator);
        assert_eq!(color, Color32::from_rgb(18, 52, 86));
        assert_eq!(body, "Salut : tout le monde");
    }
}
