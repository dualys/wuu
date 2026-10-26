use wuu::{Noun, Prompt, PromptKey, PromptResult};

fn main() {
    let mut text = Prompt::text("Commit message");
    for ch in "Fix the status output".chars() {
        text.handle_key(PromptKey::Character(ch));
    }
    text.handle_key(PromptKey::Enter);
    if let Some(PromptResult::Text(message)) = text.result() {
        println!("Commit message: {message}");
    }

    let branches = ["main", "release", "feature"];
    let mut branch = Prompt::select("Switch to branch", &branches).unwrap();
    branch.handle_key(PromptKey::Down);
    branch.handle_key(PromptKey::Enter);
    if let Some(PromptResult::Selected(index)) = branch.result() {
        println!("Selected branch: {}", branches[index]);
    }

    let actions = ["stage", "commit", "push"];
    let mut selected = [false; 3];
    {
        let mut action_prompt =
            Prompt::multi_select("Choose actions", &actions, &mut selected).unwrap();
        action_prompt.handle_key(PromptKey::Toggle);
        action_prompt.handle_key(PromptKey::Down);
        action_prompt.handle_key(PromptKey::Toggle);
        action_prompt.handle_key(PromptKey::Enter);
        if let Some(PromptResult::Multiple(choices)) = action_prompt.result() {
            println!("Selected actions: {choices:?}");
        }
    }
    println!("Selection storage: {selected:?}");

    let mut confirm = Prompt::confirm("Push the commit?");
    confirm.handle_key(PromptKey::Character('y'));
    confirm.handle_key(PromptKey::Enter);
    if let Some(PromptResult::Confirm(yes)) = confirm.result() {
        println!("Push confirmed: {yes}");
    }

    let objects = [Noun::of(b"first file"), Noun::of(b"second file")];
    let mut noun_prompt = Prompt::noun_select("Choose object", &objects)
        .unwrap()
        .noun_prefix_len(6);
    noun_prompt.handle_key(PromptKey::Down);
    noun_prompt.handle_key(PromptKey::Enter);
    if let Some(PromptResult::Noun(noun)) = noun_prompt.result() {
        println!("Selected noun: {noun:?}");
    }

    let mut rendered = String::new();
    noun_prompt.write_to(&mut rendered).unwrap();
    println!("Prompt rendering:\n{rendered}");
}
