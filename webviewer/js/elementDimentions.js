import { Graph } from "./graph/graph.js";
import { ctx } from "./griddiagram/griddiagram.js";
import { redraw } from "./griddiagram/griddiagramAnim.js";
let graphWidth = .5;
let graphHeight = 1;
let canvasSize = .4; // assumed to be a square
function resize(_) {
	Graph.width(window.innerWidth * graphWidth)
	Graph.height(window.innerHeight * graphHeight)
	ctx.canvas.width = window.innerWidth * canvasSize;
	ctx.canvas.height = window.innerWidth * canvasSize;
	redraw();


}

window.addEventListener('resize',  resize, true);
resize();
