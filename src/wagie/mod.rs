use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command {
    Decrypt { content: String, pk: String }
}

#[derive(Debug)]
pub enum Response {
    DecryptedText
}

pub struct Boss {
    from_wagie: crossbeam_channel::Receiver<Response>,
    to_wagie: mpsc::UnboundedSender<Command>,
}

impl Boss {
    pub fn init() -> Self {
        let (from_wagie_sender, from_wagie_receiver) = crossbeam_channel::unbounded::<Response>();
        let (to_wagie_sender, to_wagie_receiver) = mpsc::unbounded_channel::<Command>();

        tokio::spawn(async move {
            let mut wagie = Wagie::init(to_wagie_receiver, from_wagie_sender);
            wagie.run().await;
        });

        Self {
            from_wagie: from_wagie_receiver,
            to_wagie: to_wagie_sender,
        }
    }
}

pub struct Wagie {
    to_boss: crossbeam_channel::Sender<Response>,
    from_boss: mpsc::UnboundedReceiver<Command>,
}

impl Wagie {
    pub fn init(from_boss: mpsc::UnboundedReceiver<Command>, to_boss: crossbeam_channel::Sender<Response>) -> Self {
        Self {
            to_boss,
            from_boss,
        }
    }

    async fn run(&mut self) {
        while let Some(command) = self.from_boss.recv().await {
            match command {
                Command::Decrypt { content, pk } => {
                    // Process the decrypt command
                    // For now, just send a dummy response
                    let _ = self.to_boss.send(Response::DecryptedText);
                }
            }
        }
    }
}
