use day09 as day;

fn main() {
    let model_props = ui::ModelProps {
        input: "".to_string(),
        solve_1: day::part_1,
        solve_2: day::part_2,
    };
    yew::Renderer::<ui::Model<_, _, _, _>>::with_props(model_props).render();
}
