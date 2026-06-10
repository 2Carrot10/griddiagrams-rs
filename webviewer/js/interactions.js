// import { clickedNode, clickedLink } from "./graph/graph.js";
// import { clickLink, clickNode } from "./graph/graph.js";
function goFullScreen(){
	var canvas = document.getElementById("canvas");
	if(canvas.requestFullScreen)
		canvas.requestFullScreen();
	else if(canvas.webkitRequestFullScreen)
		canvas.webkitRequestFullScreen();
	else if(canvas.mozRequestFullScreen)
		canvas.mozRequestFullScreen();
}


// var clickLinkButton = clickLink;
// export function clickNodeButton(val) { clickNode(val); }
