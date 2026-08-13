use ollama_rs::{Ollama, generation::chat::{ChatMessage, request::ChatMessageRequest}};

pub async fn ask_nexus_brain(prompt: &str) -> Result<String, String> {
    let client = Ollama::default();
    
    let system_prompt = r#"Ты — NexusBrain, AI-агент для NEXUS-VPN. Отвечай кратко на русском языке (1-2 предложения). Твоя задача — помогать с настройкой VPN."#;

    let messages = vec![
        ChatMessage::system(system_prompt.to_string()),
        ChatMessage::user(prompt.to_string()),
    ];

    let request = ChatMessageRequest::new("qwen2.5-coder:1.5b".to_string(), messages);

    match client.send_chat_messages(request).await {
        Ok(res) => {
            // В ollama-rs 0.2 поле message является объектом ChatMessage, а не Option
            // Проверяем, не пустой ли контент
            if res.message.content.is_empty() {
                Ok("AI ответил, но сообщение пустое.".to_string())
            } else {
                Ok(res.message.content)
            }
        },
        Err(e) => {
            eprintln!("AI ошибка: {}", e);
            Ok(format!("Ошибка соединения с AI: {}. Убедитесь, что Ollama запущен (ollama serve) и модель установлена (ollama pull qwen2.5-coder:1.5b).", e))
        }
    }
}