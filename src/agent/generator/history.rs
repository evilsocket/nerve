use serde::{Deserialize, Serialize};

use crate::agent::namespaces::ToolOutput;

use super::Message;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConversationWindow {
    /// Use the history as is.
    Full,
    /// Use only the last N messages.
    LastN(usize),
    /// Only report the last message output and compress the previous ones.
    Summary,
}

impl ConversationWindow {
    pub fn parse(v: &str) -> anyhow::Result<Self> {
        match v.to_ascii_lowercase().as_str() {
            "full" => Ok(ConversationWindow::Full),
            "summary" => Ok(ConversationWindow::Summary),
            _ => {
                let n = v
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid conversation window, allowed values are: full, summary or an integer"))?;

                if n < 2 {
                    return Err(anyhow!("window size cannot be less than 2"));
                }

                Ok(ConversationWindow::LastN(n))
            }
        }
    }
}

impl std::fmt::Display for ConversationWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationWindow::Full => write!(f, "full conversation window"),
            ConversationWindow::LastN(n) => write!(f, "last {} messages", n),
            ConversationWindow::Summary => write!(f, "summary conversation window"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatHistory {
    // full list of messages as is
    messages: Vec<Message>,
    // history to be sent to the model
    agent_window: Vec<Message>,
    // strategy used to build the history for the conversation
    conversation_window_strategy: ConversationWindow,
}

impl ChatHistory {
    pub fn create(conversation: Vec<Message>, window: ConversationWindow) -> Self {
        let history = match window {
            ConversationWindow::Full => conversation.clone(),
            ConversationWindow::LastN(n) => {
                if n <= conversation.len() {
                    conversation[conversation.len() - n..].to_vec()
                } else {
                    conversation.clone()
                }
            }
            ConversationWindow::Summary => {
                let mut summarized = vec![];

                // get the index of the last Feedback message
                let last_feedback_idx = conversation
                    .iter()
                    .rposition(|m| matches!(m, Message::Feedback { .. }))
                    .unwrap_or(0);

                // all messages before the last feedback message get compressed
                for m in conversation[..last_feedback_idx].iter() {
                    summarized.push(match m {
                        Message::Agent { content, tool_call } => Message::Agent {
                            content: content.clone(),
                            tool_call: tool_call.clone(),
                        },
                        Message::Feedback { tool_call, result } => {
                            // TODO: find a more explicative message possibly hinting at the memory namespace
                            let compressed = "<output removed>";
                            Message::Feedback {
                                tool_call: tool_call.clone(),
                                result: if compressed.len() < result.to_string().len() {
                                    ToolOutput::text(compressed.to_string())
                                } else {
                                    result.clone()
                                },
                            }
                        }
                    });
                }

                // all messages from the last feedback message onwards are reported as is
                summarized.extend(conversation[last_feedback_idx..].iter().cloned());

                // the last message (feedback) is always reported as is
                summarized
            }
        };

        Self {
            conversation_window_strategy: window,
            messages: conversation,
            agent_window: history,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Message> {
        self.agent_window.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::ToolCall;

    use super::*;

    #[test]
    fn test_parse_full() {
        let window = ConversationWindow::parse("full").unwrap();
        assert!(matches!(window, ConversationWindow::Full));
    }

    #[test]
    fn test_parse_summary() {
        let window = ConversationWindow::parse("summary").unwrap();
        assert!(matches!(window, ConversationWindow::Summary));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(ConversationWindow::parse("invalid").is_err());
    }

    #[test]
    fn test_parse_case_insensitive() {
        let window = ConversationWindow::parse("FULL").unwrap();
        assert!(matches!(window, ConversationWindow::Full));

        let window = ConversationWindow::parse("Summary").unwrap();
        assert!(matches!(window, ConversationWindow::Summary));
    }

    #[test]
    fn test_parse_integer() {
        let window = ConversationWindow::parse("3").unwrap();
        assert!(matches!(window, ConversationWindow::LastN(3)));

        let window = ConversationWindow::parse("10").unwrap();
        assert!(matches!(window, ConversationWindow::LastN(10)));
    }

    #[test]
    fn test_parse_zero() {
        assert!(ConversationWindow::parse("0").is_err());
    }

    #[test]
    fn test_parse_negative() {
        assert!(ConversationWindow::parse("-1").is_err());
    }

    #[test]
    fn test_parse_invalid_integer() {
        assert!(ConversationWindow::parse("12a").is_err());
        assert!(ConversationWindow::parse("a12").is_err());
    }

    #[test]
    fn test_full_strategy_empty() {
        let history = ChatHistory::create(vec![], ConversationWindow::Full);
        assert_eq!(history.agent_window.len(), 0);
    }

    #[test]
    fn test_full_strategy_single_agent() {
        let conv = vec![Message::Agent {
            content: "test".to_string(),
            tool_call: None,
        }];
        let history = ChatHistory::create(conv.clone(), ConversationWindow::Full);
        assert_eq!(history.agent_window, conv);
    }

    #[test]
    fn test_full_strategy_agent_feedback() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("test2"),
            },
        ];
        let history = ChatHistory::create(conv.clone(), ConversationWindow::Full);
        assert_eq!(history.agent_window, conv);
    }

    #[test]
    fn test_full_strategy_multiple_messages() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                result: ToolOutput::text("test2"),
                tool_call: None,
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                result: ToolOutput::text("test4"),
                tool_call: None,
            },
        ];
        let history = ChatHistory::create(conv.clone(), ConversationWindow::Full);
        assert_eq!(history.agent_window, conv);
    }

    #[test]
    fn test_summary_strategy_empty() {
        let history = ChatHistory::create(vec![], ConversationWindow::Summary);
        assert_eq!(history.agent_window.len(), 0);
    }

    #[test]
    fn test_summary_strategy_single_agent() {
        let conv = vec![Message::Agent {
            content: "test".to_string(),
            tool_call: None,
        }];
        let history = ChatHistory::create(conv.clone(), ConversationWindow::Summary);
        assert_eq!(history.agent_window, conv);
    }

    #[test]
    fn test_summary_strategy_agent_feedback() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("test2"),
            },
        ];
        let history = ChatHistory::create(conv.clone(), ConversationWindow::Summary);
        assert_eq!(history.agent_window, conv);
    }

    #[test]
    fn test_summary_strategy_compresses_old_feedback() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("long feedback that should be compressed"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("final very very very very long feedback"),
            },
        ];

        let expected = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("<output removed>"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("final very very very very long feedback"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::Summary);
        assert_eq!(history.agent_window, expected);
    }

    #[test]
    fn test_summary_strategy_keeps_short_feedback() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("ok"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("final"),
            },
        ];

        let expected = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("ok"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("final"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::Summary);
        assert_eq!(history.agent_window, expected);
    }

    #[test]
    fn test_summary_strategy_preserves_tool_calls() {
        let tool_call = Some(ToolCall::new("test".to_string(), None, None));
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: tool_call.clone(),
            },
            Message::Feedback {
                tool_call: tool_call.clone(),
                result: ToolOutput::text("very very very very long feedback"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: tool_call.clone(),
            },
            Message::Feedback {
                tool_call: tool_call.clone(),
                result: ToolOutput::text("final"),
            },
        ];

        let expected = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: tool_call.clone(),
            },
            Message::Feedback {
                tool_call: tool_call.clone(),
                result: ToolOutput::text("<output removed>"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: tool_call.clone(),
            },
            Message::Feedback {
                tool_call: tool_call.clone(),
                result: ToolOutput::text("final"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::Summary);
        assert_eq!(history.agent_window, expected);
    }

    #[test]
    fn test_last_n_strategy() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback1"),
            },
            Message::Agent {
                content: "test2".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback2"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback3"),
            },
            Message::Agent {
                content: "test4".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback4"),
            },
        ];

        let expected = vec![
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback3"),
            },
            Message::Agent {
                content: "test4".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback4"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::LastN(4));
        assert_eq!(history.agent_window, expected);
    }

    #[test]
    fn test_last_n_strategy_with_small_conv() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback1"),
            },
        ];

        let expected = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback1"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::LastN(10));
        assert_eq!(history.agent_window, expected);
    }

    #[test]
    fn test_last_n_strategy_with_just_enough() {
        let conv = vec![
            Message::Agent {
                content: "test1".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback1"),
            },
            Message::Agent {
                content: "test2".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback2"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback3"),
            },
            Message::Agent {
                content: "test4".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback4"),
            },
        ];

        let expected = vec![
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback1"),
            },
            Message::Agent {
                content: "test2".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback2"),
            },
            Message::Agent {
                content: "test3".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback3"),
            },
            Message::Agent {
                content: "test4".to_string(),
                tool_call: None,
            },
            Message::Feedback {
                tool_call: None,
                result: ToolOutput::text("feedback4"),
            },
        ];

        let history = ChatHistory::create(conv, ConversationWindow::LastN(7));
        assert_eq!(history.agent_window, expected);
    }
}
