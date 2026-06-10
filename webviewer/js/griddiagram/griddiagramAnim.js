import { ctx, canvas } from "./griddiagram.js";
function squareAt(x, y, sizePx, rotate) {
	if (rotate) {
		[x, y] = [y, x];
	}
	ctx.fillStyle = "white";
	ctx.beginPath();
	ctx.rect(x * canvas.width - sizePx/2, y * canvas.height - sizePx/2, sizePx, sizePx);
	ctx.fill();
}

function XAt(x, y, rotate) {
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

function OAt(x, y, rotate) {
	if (rotate) {
		[x, y] = [y, x];
	}
	ctx.lineWidth = 4;
	ctx.strokeStyle = "red";

	ctx.beginPath();
	ctx.arc(x * canvas.width, y * canvas.width, 8, 0, 2 * Math.PI);
	ctx.stroke();
}

function drawLine(startX, startY, endX, endY, rotate) {
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

var currentVlist = null;

export function transitionCanvasWithVlist(vlist, delta) {
	if(currentVlist == null) return setCurrentVlist(vlist);

	let p = 0;
	let next = () => {
		p+= delta;
		p = p > 1 ? 1 : p;
		let outputvlist = vlist.map((x, i) => [x[0] * p + (currentVlist[i][0] * (1 - p)), x[1] * p + (currentVlist[i][1] * (1 - p)) ]);
		redrawVlist(outputvlist);
		if (p!=1) {
			window.requestAnimationFrame(next);
		} else {
			setCurrentVlist(vlist);
		}
	}
	window.requestAnimationFrame(next);
}

export function redrawVlist(vlist) {
	if(vlist == null) return;
	ctx.reset();
	let scale = vlist.length + 1;
	let vlistLeft = vlist.map(x => x[0]);
	let vlistRight = vlist.map(x => x[1]);

	for(let i = 0; i < vlist.length; i++) {
		let ix = vlist[i][0];;
		let start = vlistLeft.indexOf(ix);
		let end = vlistRight.indexOf(ix);

		drawLine((start + 1) / scale, (ix + 1) / scale, (end + 1) / scale, (ix + 1) / scale);
	}

	for(let i = 0; i < vlist.length; i++) {

		let segment = vlist[i];

		let start = (segment[0]);
		let end = (segment[1]);
		drawLine((i + 1) / scale, (start + 1) / scale, (i + 1) / scale, (end + 1) / scale);
	}

	for(let i = 0; i < vlist.length; i++) {
		let index = vlistLeft[i]
		squareAt((i + 1) / scale, (index + 1) / scale, 14);
		XAt((i + 1) / scale, (index + 1) / scale);
	}

	for(let i = 0; i < vlist.length; i++) {
		let index = vlistRight[i]
		squareAt((i + 1) / scale, (index + 1) / scale, 14);
		OAt((i + 1) / scale, (index + 1) / scale);
	}
}

export function redraw() {
	redrawVlist(currentVlist)
}

export function setCurrentVlist(vlist) {
	currentVlist = vlist;
	redraw();
}
