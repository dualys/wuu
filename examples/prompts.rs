use std::io::{self, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use wuu::{Prompt, PromptResult, PromptState, keyboard::linux::LinuxKeyboard};

struct RestoreCursor {
    active: bool,
}

impl RestoreCursor {
    fn hide(stdout: &mut impl Write) -> io::Result<Self> {
        execute!(stdout, Hide)?;
        Ok(Self { active: true })
    }

    fn restore(&mut self) -> io::Result<()> {
        if self.active {
            execute!(io::stdout(), Show)?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for RestoreCursor {
    fn drop(&mut self) {
        if self.active {
            let _ = execute!(io::stdout(), Show);
        }
    }
}

fn main() -> io::Result<()> {
    let branches = ["main", "release", "feature"];
    let mut prompt =
        Prompt::select("Switch to branch", &branches).expect("the example has branch choices");
    let mut keyboard = LinuxKeyboard::new()?;
    let mut stdout = io::stdout().lock();
    let mut cursor = RestoreCursor::hide(&mut stdout)?;

    let mut rendered = String::new();
    prompt
        .write_to(&mut rendered)
        .expect("writing to String cannot fail");
    stdout.write_all(rendered.as_bytes())?;
    stdout.flush()?;

    while prompt.state() == PromptState::Pending {
        if let Some(key) = keyboard.read_key(Duration::from_millis(100))? {
            prompt.handle_key(key);
            rendered.clear();
            prompt
                .write_to(&mut rendered)
                .expect("writing to String cannot fail");
            stdout.write_all(rendered.as_bytes())?;
            stdout.flush()?;
        }
    }

    keyboard.restore()?;
    cursor.restore()?;
    writeln!(stdout)?;

    match prompt.result() {
        Some(PromptResult::Selected(index)) => {
            writeln!(stdout, "Selected branch: {}", branches[index])?;
        }
        None => writeln!(stdout, "Selection cancelled.")?,
        _ => unreachable!("select prompts return an option index"),
    }

    Ok(())
}
