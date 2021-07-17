mod num_model;
mod text_model;
use num_model::NumModel;
use text_model::TextModel;

use flume::{Receiver, Selector, Sender};
use std::borrow::Cow;
use std::cell::RefCell;
use std::{io, thread};

fn main() {
    let (input_tx, input_rx) = flume::unbounded();
    let (message_tx, message_rx) = flume::unbounded();
    let (command_tx, command_rx) = flume::unbounded();

    thread::spawn(move || read_input_from_stdin(input_tx));

    thread::spawn({
        let message_tx = message_tx.clone();
        move || turn_commands_into_messages(command_rx, message_tx)
    });

    thread::spawn({
        let message_tx = message_tx.clone();
        move || num_model::auto_decrement(message_tx)
    });

    thread::spawn(move || num_model::randomize(message_tx));

    run_model_until_death(
        NumModel::default(),
        input_rx.clone(),
        message_rx,
        command_tx,
    );

    let (_message_tx, message_rx) = flume::unbounded();
    let (command_tx, _command_rx) = flume::unbounded();

    run_model_until_death(TextModel::default(), input_rx, message_rx, command_tx);
}

fn run_model_until_death<M: Model>(
    model: M,
    input_rx: Receiver<Input>,
    message_rx: Receiver<M::Message>,
    command_tx: Sender<M::Command>,
) {
    let model = RefCell::new(model);

    loop {
        let state_change = Selector::new()
            .recv(&input_rx, |input| {
                let input = input.unwrap();
                model.borrow_mut().update(input.into())
            })
            .recv(&message_rx, |message| {
                let message = message.unwrap();
                model.borrow_mut().update(message)
            })
            .wait();

        match state_change {
            StateChange::Alive {
                command: Some(command),
            } => command_tx.send(command).unwrap(),

            StateChange::Alive { command: None } => {}

            StateChange::Dead => break,
        }

        println!("{}", model.borrow().view());
    }
}

fn read_input_from_stdin(input_tx: Sender<Input>) -> ! {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = match input.trim() {
            "+" => Input::Increment,
            "-" => Input::Decrement,
            "quit" => Input::Quit,
            other => Input::Other(other.to_string()),
        };

        input_tx.send(input).unwrap();
    }
}

fn turn_commands_into_messages<M>(command_rx: Receiver<impl Command<M>>, message_tx: Sender<M>) {
    for command in command_rx {
        if let Some(message) = command.into_message() {
            message_tx.send(message).unwrap();
        }
    }
}

enum StateChange<Command> {
    Alive { command: Option<Command> },
    Dead,
}

enum Input {
    Increment,
    Decrement,
    Quit,
    Other(String),
}

trait Model {
    type Message: From<Input>;
    type Command: Command<Self::Message>;
    fn update(&mut self, message: Self::Message) -> StateChange<Self::Command>;
    fn view(&self) -> Cow<'static, str>;
}

trait Command<M> {
    fn into_message(self) -> Option<M>;
}
