use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, EnableBracketedPaste, DisableBracketedPaste},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Attribute, SetAttribute},
    terminal::{self},
};
use std::io::{self, Write};
use std::process::Command;
use tempfile::NamedTempFile;

fn main() -> io::Result<()> {
    let mut file1 = NamedTempFile::new()?;
    let mut file2 = NamedTempFile::new()?;

    capture_paste(&mut file1, "═══ Paste first content ═══")?;
    capture_paste(&mut file2, "═══ Paste second content ═══")?;

    println!();
    execute!(
        io::stdout(),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Blue),
        Print("═══ Diff ═══\n"),
        ResetColor
    )?;

    Command::new("delta")
        .arg(file1.path())
        .arg(file2.path())
        .status()?;

    Ok(())
}

fn capture_paste(file: &mut NamedTempFile, prompt: &str) -> io::Result<()> {
    println!();
    execute!(
        io::stdout(),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Blue),
        Print(prompt),
        Print("\n"),
        ResetColor,
        SetForegroundColor(Color::Cyan),
        Print("(Paste content - Enter to paste more and finish)\n"),
        ResetColor
    )?;

    execute!(io::stdout(), EnableBracketedPaste)?;
    terminal::enable_raw_mode()?;

    let mut all_content = Vec::new();

    loop {
        match event::read()? {
            Event::Paste(data) => {
                let lines = data.lines().count();
                let words = data.split_whitespace().count();
                let chars = data.len();

                terminal::disable_raw_mode()?;
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Green),
                    Print(format!("[{} lines, {} words, {} chars, pasted from clipboard]", lines, words, chars)),
                    ResetColor
                )?;
                io::stdout().flush()?;

                all_content.push(data);
                terminal::enable_raw_mode()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if !all_content.is_empty() {
                    terminal::disable_raw_mode()?;
                    execute!(io::stdout(), DisableBracketedPaste)?;
                    println!();
                    break;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                terminal::disable_raw_mode()?;
                execute!(io::stdout(), DisableBracketedPaste)?;
                println!("\n\nAborted");
                std::process::exit(0);
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;

    // Write all content to file
    for content in all_content {
        file.write_all(content.as_bytes())?;
    }
    file.flush()?;

    Ok(())
}
