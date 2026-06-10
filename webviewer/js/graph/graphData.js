const N = 30;
export const gData = {
	nodes: [...Array(N).keys()].map(i => ({ id: i })),
	links: [...Array(N - 2).keys()]
	.filter(id => id)
	.map(id => ({
		source: id,
		target: ((id) % (N)) + 1
	}))
};


// Remove this
// set `.name` of links here
/*
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
*/
