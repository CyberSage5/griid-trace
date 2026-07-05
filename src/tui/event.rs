use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    _sender: tokio::sync::mpsc::UnboundedSender<Event>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            loop {
                if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
                    if let CrosstermEvent::Key(key) = event::read().unwrap() {
                        let _ = sender.send(Event::Key(key));
                    }
                }
                // Send tick events
                let _ = sender.send(Event::Tick);
            }
        });
        
        Self {
            _sender: sender,
            receiver,
        }
    }
    
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}
