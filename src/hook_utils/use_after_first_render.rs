pub struct AfterFirstRenderState {
    rendered: bool,
}

impl AfterFirstRenderState {
    pub fn new() -> Self {
        Self { rendered: false }
    }

    pub fn mark_rendered(&mut self) {
        self.rendered = true;
    }

    pub fn has_rendered(&self) -> bool {
        self.rendered
    }
}

impl Default for AfterFirstRenderState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_after_first_render_state() {
        let mut state = AfterFirstRenderState::new();
        assert!(!state.has_rendered());

        state.mark_rendered();
        assert!(state.has_rendered());
    }
}
