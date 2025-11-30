#![deny(clippy::pedantic)]

use std::time::Duration;
use std::marker::PhantomData;
use std::fmt::Debug;

use instant::Instant;

use web_sys::HtmlInputElement;

use yew::prelude::*;

use gloo_console::log;
use gloo_worker::{HandlerId, Spawnable, Worker, WorkerBridge, WorkerScope};

pub trait Function
where Self: 'static,
{
    fn f(input: &str) -> impl Debug;
}

pub struct Solve<F>(PhantomData<F>);

impl<F: Function> Worker for Solve<F> {
    type Input = String;
    type Message = ();
    type Output = (String, Duration);

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self(PhantomData)
    }

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        let now = Instant::now();
        let result = format!("{:?}", F::f(&msg));
        let elapsed = now.elapsed();
        scope.respond(id, (result, elapsed));
    }
}

#[derive(Properties, PartialEq)]
pub struct ModelProps {
    input: String,
}

impl ModelProps {
    #[must_use]
    pub fn new(input: String) -> Self {
        Self { input }
    }
}

pub enum Msg {
    Run(String),
    Solve1(String, Duration),
    Solve2(String, Duration),
}

pub struct Model<F1: Function, F2: Function> {
    input_ref: NodeRef,
    part1: Option<String>,
    part2: Option<String>,
    input: String,
    elapsed_part_1: Option<Duration>,
    elapsed_part_2: Option<Duration>,
    bridge_solve_1: WorkerBridge<Solve<F1>>,
    bridge_solve_2: WorkerBridge<Solve<F2>>,
}

impl<F1: Function, F2: Function> Component for Model<F1, F2> {
    type Message = Msg;
    type Properties = ModelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let input = ctx.props().input.clone();

        let this = ctx.link().clone();
        let bridge_solve_1 = Solve::<F1>::spawner()
            .callback(move |(r, d)| {
                log!(format!("solve_1 {r} {d:?}"));
                this.clone().send_message(Msg::Solve1(r, d));
            })
            .spawn("./solve1.js");

        let this = ctx.link().clone();
        let bridge_solve_2 = Solve::<F2>::spawner()
            .callback(move |(r, d)| {
                log!(format!("solve_2 {r} {d:?}"));
                this.clone().send_message(Msg::Solve2(r, d));
            })
            .spawn("./solve2.js");

        Self {
            input_ref: NodeRef::default(),
            part1: None,
            part2: None,
            input,
            elapsed_part_1: None,
            elapsed_part_2: None,
            bridge_solve_1,
            bridge_solve_2,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Run(input) => {
                self.input = input;

                self.part1 = Some("Running...".to_string());
                self.part2 = Some("Running...".to_string());

                self.bridge_solve_1.send(self.input.clone());
                self.bridge_solve_2.send(self.input.clone());

                true
            }
            Msg::Solve1(result, elapsed) => {
                self.part1 = Some(result);
                self.elapsed_part_1 = Some(elapsed);
                true
            }
            Msg::Solve2(result, elapsed) => {
                self.part2 = Some(result);
                self.elapsed_part_2 = Some(elapsed);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let input_ref = self.input_ref.clone();

        let onclick = link.batch_callback(move |_| {
            let input = input_ref.cast::<HtmlInputElement>();
            input.map(|input| Msg::Run(input.value()))
        });

        html! {
            <>
                <label for="input"> { "Input: " }
            <textarea id="input" ref={self.input_ref.clone()} rows="4" cols="50" value={self.input.clone()} />
                </label>
                <button {onclick}>{ "\u{23F5}" }</button>
                <label for="results"> { "Results: " }
            <div id="results" class="output">
                <div class="result"><label> { "Part 1: " } </label> { self.part1.clone() }</div>
                <div class="result"><label> { "Part 2: " } </label> { self.part2.clone() }</div>
            </div>
            <div id="elapsed" class="output">
                <div class="result"><label> { "Part 1 Elapsed: " } </label> { format_duration(self.elapsed_part_1) }</div>
                <div class="result"><label> { "Part 2 Elapsed: " } </label> { format_duration(self.elapsed_part_2) }</div>
            </div>
            </label>
                </>
        }
    }
}

fn format_duration(elapsed: Option<Duration>) -> String {
    elapsed.map_or_else(
        || "not run".to_string(),
        |v| format!("{}ms ({}us)", v.as_millis(), v.as_micros()),
    )
}
