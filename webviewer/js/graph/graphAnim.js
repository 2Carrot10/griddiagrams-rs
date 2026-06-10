import { highlightNodes, getNeighborNodes, getNeighborLinks, highlightLinks, hoverNode, hoverLink, clickedNode, clickedLink, Graph } from "./graph.js";

// removal
function setCurrentNode(name) {
	document.getElementById("selection-text").innerText = name;
}

function setCurrentLink(name) {
	document.getElementById("selection-text").innerText = name;
}

function addHighlightLinkNeighbors(link) {
	if (link) {
		highlightLinks.add(link);
		highlightNodes.add(link.source);
		highlightNodes.add(link.target);
	}

};
function addHighlightNodeNeighbors(node) {
	if (node) {
		highlightNodes.add(node);
		getNeighborNodes(node).forEach(neighbor => highlightNodes.add(neighbor));
		getNeighborLinks(node).forEach(link => highlightLinks.add(link));
	}

};

export function updateHighlight() {

	highlightNodes.clear();
	highlightLinks.clear();
	addHighlightNodeNeighbors(hoverNode);
	addHighlightLinkNeighbors(hoverLink);

	addHighlightNodeNeighbors(clickedNode);
	addHighlightLinkNeighbors(clickedLink);

	Graph
		.nodeColor(Graph.nodeColor())
		.linkWidth(Graph.linkWidth())
		.linkDirectionalParticles(Graph.linkDirectionalParticles());
}
