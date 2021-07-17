use flume::Sender;
use rand::Rng;
use std::borrow::Cow;
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct NumModel {
    n: i32,
    saw_other: bool,
}

pub(crate) enum Message {
    Input(crate::Input),
    Randomize(i32),
}

impl From<crate::Input> for Message {
    fn from(input: crate::Input) -> Self {
        Self::Input(input)
    }
}

pub(crate) enum Command {
    MaybeIncrement,
}

impl crate::Command<Message> for Command {
    fn into_message(self) -> Option<Message> {
        match self {
            Command::MaybeIncrement => rand::thread_rng()
                .gen::<bool>()
                .then(|| Message::Input(crate::Input::Increment)),
        }
    }
}

impl crate::Model for NumModel {
    type Message = Message;
    type Command = Command;

    fn update(&mut self, message: Self::Message) -> crate::StateChange<Self::Command> {
        match message {
            Message::Input(input) => match input {
                crate::Input::Increment => {
                    self.saw_other = false;
                    self.n += 1;
                }

                crate::Input::Decrement => {
                    self.saw_other = false;
                    self.n -= 1;
                }

                crate::Input::Quit => return crate::StateChange::Dead,

                crate::Input::Other(_) => self.saw_other = true,
            },

            Message::Randomize(n) => self.n = n,
        }

        if self.n == 5 {
            return crate::StateChange::Alive {
                command: Some(Command::MaybeIncrement),
            };
        }

        crate::StateChange::Alive { command: None }
    }

    fn view(&self) -> Cow<'static, str> {
        if self.saw_other {
            Cow::Borrowed("What was that?")
        } else {
            Cow::Owned(self.n.to_string())
        }
    }
}

pub(crate) fn auto_decrement(message_tx: Sender<Message>) {
    loop {
        thread::sleep(Duration::from_secs(2));

        if message_tx
            .send(Message::Input(crate::Input::Decrement))
            .is_err()
        {
            return;
        }
    }
}

pub(crate) fn randomize(message_tx: Sender<Message>) {
    loop {
        thread::sleep(Duration::from_secs(5));

        if message_tx
            .send(Message::Randomize(rand::thread_rng().gen_range(1..=10)))
            .is_err()
        {
            return;
        }
    }
}
