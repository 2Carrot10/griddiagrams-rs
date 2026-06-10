import { gData } from "./graphData.js";
import { updateHighlight } from "./graphAnim.js";
export const highlightNodes = new Set();
export const highlightLinks = new Set();
export let hoverNode = null; export let hoverLink = null;
export let clickedNode = null; export let clickedLink = null;

export const Graph = new ForceGraph3D(document.getElementById('3d-graph'))
	.cooldownTicks(100)
	.graphData(gData)
	.nodeColor(node => highlightNodes.has(node) ? node === hoverNode || node === clickedNode ? 'rgb(255,0,0,1)' : 'rgba(255,160,0,0.8)' : 'rgba(0,255,255,0.6)')
	.linkWidth(link => highlightLinks.has(link) ? 4 : 1)
	.onNodeHover(node => {
		// no state change
		if ((!node && !highlightNodes.size) || (node && hoverNode === node)) return;

		hoverNode = node || null;

		updateHighlight();
	})
	.onLinkHover(link => {
		hoverLink = link || null;
		updateHighlight();
	})
	.onLinkClick(clickLink)
	.onNodeClick(clickNode)
	.enableNodeDrag(false)
	.backgroundColor("#0B2027");

export function clickNode(node) {
	clickedNode = node || null;
	clickedLink = null;
	putNextNodes(getNeighborNodes(node).map(x => x.id))

	// setCurrentNode(node.id);
	updateHighlight();
}

export function clickLink(link) {
	clickedLink = link || null;
	clickedNode = null;
	// setCurrentLink(link.name); // link.source.id, link.target.id);
	updateHighlight();
}


function getNodeFromId(id) {
	return gData.nodes.filter((x) => x.id == id)[0];
}

export function getNeighborNodes(id) {
	return getOutwardNeighborLinks(id).map(x => x.target).concat(getInwardNeighborLinks(id).map(x => x.source));
}

export function getNeighborLinks(id) {
	return getOutwardNeighborLinks(id).concat(getInwardNeighborLinks(id));
}

export function getOutwardNeighborLinks(id) {
	return gData.links.filter((x) => x.source == id);
}

export function getInwardNeighborLinks(id) {
	return gData.links.filter((x) => x.target == id);
}

// move location
const next_nodes = document.getElementById("next-nodes");
function putNextNodes(contents) {
	next_nodes.innerHTML = contents.map((x, i) =>
		`<li><button class="button">${x}</button></li>`
	).join('');

	next_nodes.querySelectorAll("button").forEach((button, i) => {
		button.addEventListener("click", () => clickNode(getNodeFromId(contents[i])));
	});
}
