use eframe::egui::Color32;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

pub struct Line(pub String, pub Color32, pub String);

fn message(raw: &str) -> Option<Line> {
    let (header, body) = raw.split_once(" PRIVMSG ")?;
    let (_, body) = body.split_once(" :")?;
    let mut name = header
        .split_whitespace()
        .last()?
        .trim_start_matches(':')
        .split('!')
        .next()?
        .to_owned();
    let mut color = Color32::LIGHT_BLUE;
    let mut badges = Vec::new();
    if let Some(tags) = header.strip_prefix('@') {
        for field in tags.split_whitespace().next()?.split(';') {
            if let Some(value) = field
                .strip_prefix("display-name=")
                .filter(|v| !v.is_empty())
            {
                name = value.to_string();
            }
            if let Some(value) = field
                .strip_prefix("color=#")
                .and_then(|v| u32::from_str_radix(v, 16).ok())
            {
                color = Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8);
            }
            if let Some(value) = field.strip_prefix("badges=") {
                badges.extend(
                    value
                        .split(',')
                        .filter_map(|b| b.split_once('/').map(|p| p.0)),
                );
            }
        }
    }
    if !badges.is_empty() {
        name = format!("[{}] {name}", badges.join(", "));
    }
    Some(Line(name, color, body.to_string()))
}

pub fn start(
    channel: String,
    retry: bool,
    delay: u64,
    tx: Sender<Line>,
    stop: std::sync::mpsc::Receiver<()>,
) {
    thread::spawn(move || {
        let status = |s: String| tx.send(Line("Twitch".into(), Color32::GRAY, s)).is_ok();
        loop {
            let run = || -> Result<(), Box<dyn std::error::Error>> {
                let tcp = TcpStream::connect("irc.chat.twitch.tv:6697")?;
                tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
                tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
                let mut tls =
                    native_tls::TlsConnector::new()?.connect("irc.chat.twitch.tv", tcp)?;
                tls.write_all(format!("CAP REQ :twitch.tv/tags twitch.tv/commands\r\nNICK justinfan{}\r\nJOIN #{channel}\r\n",std::process::id()).as_bytes())?;
                let mut reader = BufReader::new(tls);
                let mut raw = String::new();
                while matches!(stop.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)) {
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
                    } else if line.contains(" JOIN #") {
                        status(format!("Connect� � #{channel}"));
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
            if !status(
                result
                    .err()
                    .map_or_else(|| "D�connect�".into(), |e| e.to_string()),
            ) || !retry
            {
                break;
            }
            if !matches!(
                stop.recv_timeout(Duration::from_millis(delay)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_tags_and_preserves_body() {
        let Line(name,color,body)=message("@badges=moderator/1;color=#123456;display-name=Bougli :bougli!user PRIVMSG #demo :Salut : tout le monde").unwrap();
        assert_eq!(name, "[moderator] Bougli");
        assert_eq!(color, Color32::from_rgb(18, 52, 86));
        assert_eq!(body, "Salut : tout le monde");
    }
}
