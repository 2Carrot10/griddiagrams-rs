const highlightNodes = new Set();
const highlightLinks = new Set();
let hoverNode = null;
let hoverLink = null;

let clickNode = null;
let clickLink = null;
// Random tree
const N = 30;

/*
const gData = {
	nodes: [...Array(N).keys()].map(i => ({ id: i })),
	links: [...Array(N * 10).keys()]
	.filter(id => id)
	.map(id => ({
		source: id % (N - 1),
		target: Math.round(Math.random() * ((id % (N -1))))
	}))
};
*/


setCurrentNode = function(name) {
	document.getElementById("selection-text").innerText = name;
}

setCurrentLink = function(from, to) {
	document.getElementById("selection-text").innerText = from + "->"+ to;
}

const gData = {
	nodes: [...Array(N).keys()].map(i => ({ id: i })),
	links: [...Array(N - 2).keys()]
	.filter(id => id)
	.map(id => ({
		source: id,
		target: ((id) % (N)) + 1
	}))
};

gData.links.forEach(link => {
	const a = gData.nodes[link.source];
	const b = gData.nodes[link.target];
	!a.neighbors && (a.neighbors = []);
	!b.neighbors && (b.neighbors = []);
	a.neighbors.push(b);
	b.neighbors.push(a);

	!a.links && (a.links = []);
	!b.links && (b.links = []);
	a.links.push(link);
	b.links.push(link);
});



addHighlightLinkNeighbors = function(link) {
	if (link) {
		highlightLinks.add(link);
		highlightNodes.add(link.source);
		highlightNodes.add(link.target);
	}

};
addHighlightNodeNeighbors = function(node) {
	if (node) {
		highlightNodes.add(node);
		node.neighbors.forEach(neighbor => highlightNodes.add(neighbor));
		node.links.forEach(link => highlightLinks.add(link));
	}

};

const Graph = new ForceGraph3D(document.getElementById('3d-graph'))
	.cooldownTicks(100)
	.graphData(gData)
	.nodeColor(node => highlightNodes.has(node) ? node === hoverNode || node === clickNode ? 'rgb(255,0,0,1)' : 'rgba(255,160,0,0.8)' : 'rgba(0,255,255,0.6)')
	.linkWidth(link => highlightLinks.has(link) ? 4 : 1)
// .linkDirectionalParticles(link => highlightLinks.has(link) ? 4 : 0)
// .linkDirectionalParticleWidth(4)
	.onNodeHover( node => {
		// no state change
		if ((!node && !highlightNodes.size) || (node && hoverNode === node)) return;

		hoverNode = node || null;

		updateHighlight();
	})
	.onLinkHover(link => {
		hoverLink = link || null;
		updateHighlight();
	})
	.onLinkClick(link => {

		clickLink = link || null;
		clickNode = null;
		setCurrentLink(link.source.id, link.target.id);
		updateHighlight();
	})
	.onNodeClick(node => {
		clickNode = node || null;
		clickLink = null;
		putNextNodes(node.neighbors.map(x => x.id))

		setCurrentNode(node.id);
		updateHighlight();
	});

function updateHighlight() {

	highlightNodes.clear();
	highlightLinks.clear();
	addHighlightNodeNeighbors(hoverNode);
	addHighlightLinkNeighbors(hoverLink);

	addHighlightNodeNeighbors(clickNode);
	addHighlightLinkNeighbors(clickLink);

	Graph
		.nodeColor(Graph.nodeColor())
		.linkWidth(Graph.linkWidth())
		.linkDirectionalParticles(Graph.linkDirectionalParticles());
}

Graph.enableNodeDrag(false);

graphWidth = .5;
graphHeight = 1;
backgroundColor = "#0B2027";
Graph.backgroundColor(backgroundColor);
resize = function(_) {
	Graph.width(window.innerWidth * graphWidth)
	Graph.height(window.innerHeight * graphHeight)
}
window.addEventListener('resize',  resize, true);
resize();

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

squareAt = function(x, y, sizePx, rotate) {
	if (rotate) {
		[x, y] = [y, x];
	}
	ctx.fillStyle = "white";
	ctx.beginPath();
	ctx.rect(x * canvas.width - sizePx/2, y * canvas.height - sizePx/2, sizePx, sizePx);
	ctx.fill();
}

XAt = function(x, y, rotate) {
	if (rotate) {
		[x, y] = [y, x];
	}
	ctx.lineWidth = 4;
	let size = 8;
	ctx.strokeStyle = "blue";

	ctx.beginPath();
	ctx.moveTo(x * canvas.width - size, y * canvas.height - size);
	ctx.lineTo(x * canvas.width + size, y * canvas.height + size);
	ctx.stroke();

	ctx.beginPath();
	ctx.moveTo(x * canvas.width + size, y * canvas.height - size);
	ctx.lineTo(x * canvas.width - size, y * canvas.height + size);
	ctx.stroke();
}

OAt = function(x, y, rotate) {
	if (rotate) {
		[x, y] = [y, x];
	}
	ctx.lineWidth = 4;
	ctx.strokeStyle = "red";

	ctx.beginPath();
	ctx.arc(x * canvas.width, y * canvas.width, 8, 0, 2 * Math.PI);
	ctx.stroke();
}

drawLine = function(startX, startY, endX, endY, rotate) {
	if (rotate) {
		[startX, startY] = [startY, startX];
		[endX, endY] = [endY, endX];
	}
	ctx.strokeStyle = "white";
	ctx.lineWidth = 14;

	ctx.beginPath();
	ctx.moveTo(startX * canvas.width, startY * canvas.height);
	ctx.lineTo(endX * canvas.width, endY * canvas.height);
	ctx.stroke();

	ctx.beginPath();
	ctx.strokeStyle = "black";
	ctx.lineWidth = 6;
	ctx.moveTo(startX * canvas.width, startY * canvas.height);
	ctx.lineTo(endX * canvas.width, endY * canvas.height);
	ctx.stroke();
}

trasitionCanvasWithVlist = function(vlist1, vlist2, delta, rotate) {
	var p = 0;
	next = () => {
		p+= delta;
		p = p > 1 ? 1 : p;
		outputvlist = vlist1.map((x, i) => [x[0] * p + (vlist2[i][0] * (1 - p)), x[1] * p + (vlist2[i][1] * (1 - p)) ]);
		setCanvasWithVlist(outputvlist, rotate);
		if (p!=1) {
			window.requestAnimationFrame(next);
		}
	}
	window.requestAnimationFrame(next);
}

setCanvasWithVlist = function(vlist, rotate) {
	ctx.reset();
	scale = vlist.length + 1;
	vlistLeft = vlist.map(x => x[0]);
	vlistRight = vlist.map(x => x[1]);

	for(let i = 0; i < vlist.length; i++) {
		let ix = vlist[i][0];;
		start = vlistLeft.indexOf(ix);
		end = vlistRight.indexOf(ix);

		drawLine((start + 1) / scale, (ix + 1) / scale, (end + 1) / scale, (ix + 1) / scale, rotate);
	}

	for(let i = 0; i < vlist.length; i++) {

		let segment = vlist[i];

		start = (segment[0]);
		end = (segment[1]);
		drawLine((i + 1) / scale, (start + 1) / scale, (i + 1) / scale, (end + 1) / scale, rotate);
	}

	for(let i = 0; i < vlist.length; i++) {
		index = vlistLeft[i]
		squareAt((i + 1) / scale, (index + 1) / scale, 14, rotate);
		XAt((i + 1) / scale, (index + 1) / scale, rotate);
	}

	for(let i = 0; i < vlist.length; i++) {
		index = vlistRight[i]
		squareAt((i + 1) / scale, (index + 1) / scale, 14, rotate);
		OAt((i + 1) / scale, (index + 1) / scale, rotate);
	}
}
var currentVlist = [[7, 3], [1, 4], [2, 6], [3, 10], [10, 9], [8, 0], [9, 5], [4, 7], [6, 1], [5, 8], [0, 2]];
var nextVlist = [[7, 3], [1, 4], [8, 6], [3, 10], [10, 9], [2, 0], [9, 5], [4, 7], [6, 1], [5, 2], [0, 8]];
setCanvasWithVlist(currentVlist);
trasitionCanvasWithVlist(currentVlist, nextVlist, .01, true);

function goFullScreen(){
	var canvas = document.getElementById("canvas");
	if(canvas.requestFullScreen)
		canvas.requestFullScreen();
	else if(canvas.webkitRequestFullScreen)
		canvas.webkitRequestFullScreen();
	else if(canvas.mozRequestFullScreen)
		canvas.mozRequestFullScreen();
}


function putNextNodes(contents) {
	document.getElementById("next-nodes").innerHTML = contents.map((x) => {
		return `<li><button class="button">${ x } </button> </li>`;
	}).join('');

	// Alternative using arrow function expression:
	// document.getElementById('list').innerHTML = persons.map(person => `<li>${ getFullName(person) }</li>`).join('');

}
