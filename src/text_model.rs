use std::borrow::Cow;

#[derive(Default)]
pub(crate) struct TextModel {
    c: char,
}

pub(crate) enum Message {
    Input(crate::Input),
}

impl From<crate::Input> for Message {
    fn from(input: crate::Input) -> Self {
        Self::Input(input)
    }
}

pub(crate) enum Command {}

impl crate::Command<Message> for Command {
    fn into_message(self) -> Option<Message> {
        match self {}
    }
}

impl crate::Model for TextModel {
    type Message = Message;
    type Command = Command;

    fn update(&mut self, message: Self::Message) -> crate::StateChange<Self::Command> {
        match message {
            Message::Input(input) => match input {
                crate::Input::Increment => self.c = char::from_u32(self.c as u32 + 1).unwrap(),

                crate::Input::Decrement => self.c = char::from_u32(self.c as u32 - 1).unwrap(),

                crate::Input::Quit => return crate::StateChange::Dead,

                crate::Input::Other(other) => {
                    if let Some(c) = other.chars().next() {
                        self.c = c;
                    }
                }
            },
        }

        crate::StateChange::Alive { command: None }
    }

    fn view(&self) -> Cow<'static, str> {
        Cow::Owned(self.c.to_string())
    }
}
