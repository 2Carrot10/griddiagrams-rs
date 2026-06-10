import { transitionCanvasWithVlist, setCurrentVlist } from "./griddiagram/griddiagramAnim.js" ;
var currentVlist = [[7, 3], [1, 4], [2, 6], [3, 10], [10, 9], [8, 0], [9, 5], [4, 7], [6, 1], [5, 8], [0, 2]];
var nextVlist = [[7, 3], [1, 4], [8, 6], [3, 10], [10, 9], [2, 0], [9, 5], [4, 7], [6, 1], [5, 2], [0, 8]];
setCurrentVlist(currentVlist);
transitionCanvasWithVlist(nextVlist, .01);
