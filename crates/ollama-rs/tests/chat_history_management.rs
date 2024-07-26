use ollama_rs::Ollama;

#[test]
fn test_chat_history_saved_as_should() {
    let mut ollama = Ollama::new_default_with_history(30);
    let chat_id = "default";

    ollama.add_user_response(chat_id, "Hello");
    ollama.add_assistant_response(chat_id, "Hi");

    ollama.add_user_response(chat_id, "Tell me 'hi' again");
    ollama.add_assistant_response(chat_id, "Hi again");

    let history = ollama.get_messages_history(chat_id).unwrap();

    assert_eq!(history.len(), 4);

    let last = history.last();
    assert!(last.is_some());
    assert_eq!(last.unwrap().content, "Hi again".to_string());
}

#[test]
fn chat_history_not_stored_if_no_content() {
    let mut ollama = Ollama::new_default_with_history(30);
    let chat_id = "default";

    ollama.add_user_response(chat_id, "Hello");
    ollama.add_assistant_response(chat_id, "");

    ollama.add_user_response(chat_id, "");
    ollama.add_assistant_response(chat_id, "Hi again");

    let history = ollama.get_messages_history(chat_id).unwrap();

    assert_eq!(history.len(), 2);

    let last = history.last();
    assert!(last.is_some());
    assert_eq!(last.unwrap().content, "Hi again".to_string());
}

#[test]
fn clear_chat_history_for_one_id_only() {
    let mut ollama = Ollama::new_default_with_history(30);
    let first_chat_id = "default";

    ollama.add_user_response(first_chat_id, "Hello");

    let another_chat_id = "not_default";

    ollama.add_user_response(another_chat_id, "Hello");

    assert_eq!(ollama.get_messages_history(first_chat_id).unwrap().len(), 1);
    assert_eq!(
        ollama.get_messages_history(another_chat_id).unwrap().len(),
        1
    );

    ollama.clear_messages_for_id(first_chat_id);

    assert!(ollama.get_messages_history(first_chat_id).is_none());
    assert_eq!(
        ollama.get_messages_history(another_chat_id).unwrap().len(),
        1
    );
}

#[test]
fn clear_chat_history_for_all() {
    let mut ollama = Ollama::new_default_with_history(30);
    let first_chat_id = "default";

    ollama.add_user_response(first_chat_id, "Hello");

    let another_chat_id = "not_default";

    ollama.add_user_response(another_chat_id, "Hello");

    assert_eq!(ollama.get_messages_history(first_chat_id).unwrap().len(), 1);
    assert_eq!(
        ollama.get_messages_history(another_chat_id).unwrap().len(),
        1
    );

    ollama.clear_all_messages();

    assert!(ollama.get_messages_history(first_chat_id).is_none());
    assert!(ollama.get_messages_history(another_chat_id).is_none());
}
