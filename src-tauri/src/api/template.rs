use crate::log_template;
use minijinja::{context, Environment, Value};

pub fn render_prompt(
    chat_template: &Option<String>,
    messages: Vec<super::ChatMsgDto>,
) -> Result<String, String> {
    let tpl = match chat_template {
        Some(s) if !s.trim().is_empty() => s.clone(),
        _ => return Err("chat_template not available".into()),
    };

    // Лог на вход
    log_template!("render: msgs={}, tpl_len={}", messages.len(), tpl.len());
    let msgs = messages;
    let mut env = Environment::new();
    env.add_template("tpl", &tpl).map_err(|e| e.to_string())?;
    let tmpl = env.get_template("tpl").map_err(|e| e.to_string())?;

    // minijinja контекст
    let msgs_val: Vec<Value> = msgs.iter().map(Value::from_serialize).collect();
    let rendered = tmpl
        .render(context! { messages => msgs_val, add_generation_prompt => true, tools => Vec::<String>::new() })
        .map_err(|e| e.to_string())?;

    log_template!(
        "render ok, prefix=<<<{}>>>",
        rendered.chars().take(120).collect::<String>()
    );
    Ok(rendered)
}
