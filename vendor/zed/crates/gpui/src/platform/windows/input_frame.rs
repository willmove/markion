//! One queued input frame per window; no timer or synchronous rendering.
#[derive(Default)]
pub(crate) struct InputFrameRequest {
    pending: bool,
}

impl InputFrameRequest {
    pub(crate) fn request<E>(&mut self, post: impl FnOnce() -> Result<(), E>) -> Result<(), E> {
        if !self.pending {
            post()?;
            self.pending = true;
        }
        Ok(())
    }

    pub(crate) fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_input_burst_posts_only_one_frame() {
        let mut request = InputFrameRequest::default();
        let mut posts = 0;
        for _ in 0..10_000 {
            request
                .request(|| {
                    posts += 1;
                    Ok::<_, ()>(())
                })
                .unwrap();
        }
        assert_eq!(posts, 1);
        assert!(request.take());
        assert!(!request.take());
    }

    #[test]
    fn failed_post_does_not_suppress_the_next_input() {
        let mut request = InputFrameRequest::default();
        assert_eq!(request.request(|| Err("queue full")), Err("queue full"));
        request.request(|| Ok::<_, ()>(())).unwrap();
        assert!(request.take());
    }

    #[test]
    fn input_during_a_frame_can_queue_the_next_frame() {
        let mut request = InputFrameRequest::default();
        request.request(|| Ok::<_, ()>(())).unwrap();
        assert!(request.take());
        request.request(|| Ok::<_, ()>(())).unwrap();
        assert!(request.take());
        assert!(!request.take());
    }
}
