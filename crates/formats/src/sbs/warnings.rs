use super::constant::MAX_WARNINGS;
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Warnings {
    pub(super) lines: Vec<(String, usize)>,
    pub(super) line_index: HashMap<String, usize>,
    /// Messages over the limit: they are counted but never stored (audit: MaterialCraft #2).
    dropped: usize,
}

impl Warnings {
    pub(super) fn add(&mut self, message: impl Into<String>) {
        let message = message.into();
        if let Some(line) = self.line_index.get(&message).and_then(|&i| self.lines.get_mut(i)) {
            line.1 += 1;
            return;
        }

        if self.lines.len() >= MAX_WARNINGS {
            self.dropped += 1;
            return;
        }

        self.line_index.insert(message.clone(), self.lines.len());
        self.lines.push((message, 1));
    }

    pub(super) fn finish(self) -> Vec<String> {
        let total_messages = self.lines.len() + self.dropped;
        let mut messages: Vec<String> =
            self.lines.into_iter().take(MAX_WARNINGS).map(|(line, count)| if count > 1 { format!("{line} (×{count})") } else { line }).collect();
        if total_messages > MAX_WARNINGS {
            messages.push(format!("…and {} more warnings", total_messages - MAX_WARNINGS));
        }

        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_messages_are_capped_as_they_arrive() {
        let mut warnings = Warnings::default();

        for i in 0..MAX_WARNINGS + 5 {
            warnings.add(format!("warning {i}"));
        }

        assert_eq!(warnings.lines.len(), MAX_WARNINGS);

        let messages = warnings.finish();

        assert_eq!(messages.len(), MAX_WARNINGS + 1);
        assert_eq!(messages.last().unwrap(), "…and 5 more warnings");
    }
}
