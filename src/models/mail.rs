/// Email sending via sendmail, mirroring src/misc/mail.cpp.
use std::process::{Command, Stdio};
use std::io::Write;

pub fn send(to: &str, subject: &str, body: &str) {
    let date = chrono::Utc::now().format("%d %b %Y %H:%M %z").to_string();
    let to = to.to_string();
    let message = format!(
        "Date: {date}\n\
         To: <{to}>\n\
         From: Manemix <manemix@manemix.org>\n\
         Reply-To: Manemix <manemix@manemix.org>\n\
         Subject: {subject}\n\
         Content-Type: text/plain\n\n\
         {body}"
    );

    std::thread::spawn(move || {
        match Command::new("sendmail")
            .arg(&to)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(message.as_bytes());
                }
                let _ = child.wait();
            }
            Err(e) => {
                tracing::error!("sendmail error: {}", e);
            }
        }
    });
}
