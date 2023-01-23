import * as wasm from "mcmf-wasm";

wasm.init();
const builder = new wasm.GraphBuilder();
builder.add_edge("a", "b", 10, 200);
builder.add_edge("b", "c", 20, 0);
builder.add_edge("c", "e", 15, 0);
builder.add_edge("a", "d", 2, 100);
builder.add_edge("d", "e", 3, 0);

const solution = builder.solve_mcmf("a", "e");
let result = "";
result += `Max flow: ${solution.max_flow()}\n`;
result += `Total cost: ${solution.total_cost()}\n`;
result += "\n";
result += `Paths:\n`;
for (const path of solution.paths()) {
    result += path.nodes().join(" → ") + "\n";
}
document.getElementById("answer").textContent = result;
