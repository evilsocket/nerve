use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub empty_responses: usize,
    pub unparsed_responses: usize,
    pub unknown_tool_calls: usize,
    pub invalid_tool_calls: usize,
    pub errored_tool_calls: usize,
    pub timedout_tool_calls: usize,
}

impl ErrorMetrics {
    fn has_response_errors(&self) -> bool {
        self.empty_responses > 0 || self.unparsed_responses > 0
    }

    fn has_tool_errors(&self) -> bool {
        self.unknown_tool_calls > 0 || self.invalid_tool_calls > 0 || self.errored_tool_calls > 0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub last_input_tokens: u32,
    pub last_output_tokens: u32,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub max_steps: usize,
    pub current_step: usize,
    pub valid_responses: usize,
    pub valid_tool_calls: usize,
    pub success_tool_calls: usize,
    pub errors: ErrorMetrics,
    pub usage: Usage,
}

impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "step:")?;
        if self.max_steps > 0 {
            write!(f, "{}/{} ", self.current_step, self.max_steps)?;
        } else {
            write!(f, "{} ", self.current_step)?;
        }

        if self.errors.has_response_errors() {
            write!(
                f,
                "responses(valid:{} empty:{} broken:{}) ",
                self.valid_responses, self.errors.empty_responses, self.errors.unparsed_responses
            )?;
        } else if self.valid_responses > 0 {
            write!(f, "responses:{} ", self.valid_responses)?;
        }

        if self.errors.has_tool_errors() {
            write!(
                f,
                "tool_calls(valid:{} ok:{} errored:{} unknown:{} invalid:{}) ",
                self.valid_tool_calls,
                self.success_tool_calls,
                self.errors.errored_tool_calls,
                self.errors.unknown_tool_calls,
                self.errors.invalid_tool_calls
            )?;
        } else if self.valid_tool_calls > 0 {
            write!(f, "tool_calls:{} ", self.valid_tool_calls,)?;
        }

        if self.usage.last_input_tokens > 0 {
            write!(
                f,
                "token_usage(in:{} out:{} tot_in:{} tot_out:{}) ",
                self.usage.last_input_tokens,
                self.usage.last_output_tokens,
                self.usage.total_input_tokens,
                self.usage.total_output_tokens
            )?;
        }

        Ok(())
    }
}
