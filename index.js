/*
 * ATTENTION: The "eval" devtool has been used (maybe by default in mode: "development").
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
/******/ (() => { // webpackBootstrap
/******/ 	// runtime can't be in strict mode because a global variable is assign and maybe created.
/******/ 	var __webpack_modules__ = ({

/***/ "./src/index.js":
/*!**********************!*\
  !*** ./src/index.js ***!
  \**********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _plotter__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./plotter */ \"./src/plotter.js\");\n\n\n// Get references to HTML elements\nconst codeInput = document.getElementById(\"code-input\");\nconst runButton = document.getElementById(\"run-button\");\nconst outputContainer = document.getElementById(\"output-container\");\nconst outputCanvas = document.getElementById(\"output-canvas\");\nconst logOutput = document.getElementById(\"output-log\");\n\n// Set canvas resolution equal to container surface\noutputCanvas.width = outputContainer.offsetWidth;\noutputCanvas.height = outputContainer.offsetHeight;\n\n// Statevector plotter\nconst plotter = new _plotter__WEBPACK_IMPORTED_MODULE_0__.StatevectorPlotter(outputCanvas);\n\n// Initialize web worker for simulator\nconst worker = new Worker(new URL(/* worker import */ __webpack_require__.p + __webpack_require__.u(\"src_worker_js\"), __webpack_require__.b));\n\n// Add message listener to worker\nworker.addEventListener(\"message\", event => {\n  const message = event.data;\n\n  switch (message.type) {\n    case \"simulationComplete\":\n      // Draw the state vector\n      plotter.plot(message.result.statevector);\n\n      // Display times\n      const times = Object.entries(message.result.times).map(\n        ([key, value]) => `${key}: ${value.duration} ms`\n      ).join(\"\\n\");\n      logOutput.innerText = times;\n\n      // Re-enable the run button\n      runButton.removeAttribute(\"disabled\");\n      break;\n\n    case \"simulationError\":\n      console.error(\"Simulation error:\", message.error);\n\n      // Display error message\n      logOutput.innerText = message.error;\n\n      // Re-enable the run button\n      runButton.removeAttribute(\"disabled\");\n      break;\n\n    default:\n      console.warn(\"Unknown message type:\", message.type);\n      break;\n  }\n});\n\n// Add event listener to run button\nrunButton.addEventListener(\"click\", () => {\n  const code = codeInput.value;\n\n  // Disable the run button\n  runButton.setAttribute(\"disabled\", true);\n\n  // Send OpenQASM code to web worker for simulation\n  worker.postMessage({ type: \"simulate\", code: code });\n});\n\n\n//# sourceURL=webpack://SandBox/./src/index.js?");

/***/ }),

/***/ "./src/plotter.js":
/*!************************!*\
  !*** ./src/plotter.js ***!
  \************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"StatevectorPlotter\": () => (/* binding */ StatevectorPlotter)\n/* harmony export */ });\n\nclass StatevectorPlotter {\n\n  constructor(canvas) {\n    this._canvas = canvas;\n    this._ctx = canvas.getContext(\"2d\");\n    this._paddingTop = 40;\n    this._paddingRight = 50;\n    this._paddingBottom = 50;\n    this._paddingLeft = 50;\n  }\n\n  setPadding({ top, right, bottom, left }) {\n    this._paddingTop = top ?? this._paddingTop;\n    this._paddingRight = right ?? this._paddingRight;\n    this._paddingBottom = bottom ?? this._paddingBottom;\n    this._paddingLeft = left ?? this._paddingLeft;\n  }\n\n  plot(statevector) {\n    const { qubitWidth, bases } = statevector;\n    this._computeChartDimensions(qubitWidth);\n    this._clearCanvas();\n    this._drawAxes(qubitWidth);\n    this._drawAmpitudes(statevector);\n    this._drawPhases(statevector);\n  }\n\n  _computeChartDimensions(qubitWidth) {\n    this._canvasWidth = this._canvas.width;\n    this._canvasHeight = this._canvas.height;\n    this._chartWidth = this._canvasWidth - this._paddingLeft - this._paddingRight;\n    this._chartHeight = this._canvasHeight - this._paddingTop - this._paddingBottom;\n    this._chartTop = this._paddingTop;\n    this._chartBottom = this._canvasHeight - this._paddingBottom;\n    this._chartLeft = this._paddingLeft;\n    this._chartRight = this._canvasWidth - this._paddingRight;\n    this._chartMiddle = this._chartHeight / 2 + this._chartTop;\n    this._dx = this._chartWidth / (2 ** qubitWidth);\n    this._dy = this._chartHeight / 2;\n  }\n\n  _clearCanvas() {\n    this._ctx.clearRect(0, 0, this._canvasWidth, this._canvasHeight);\n  }\n\n  _drawAxes(qubitWidth) {\n    const ctx = this._ctx;\n    const {\n      _chartTop: chartTop,\n      _chartBottom: chartBottom,\n      _chartLeft: chartLeft,\n      _chartRight: chartRight,\n      _dx: dx,\n      _dy: dy,\n      _chartMiddle: middleChartHeight,\n    } = this;\n\n    // Draw X-axis\n    ctx.strokeStyle = \"#000\";\n    ctx.beginPath();\n    ctx.moveTo(chartLeft, middleChartHeight);\n    ctx.lineTo(chartRight, middleChartHeight);\n    ctx.stroke();\n\n    // Draw Y-axes\n    ctx.beginPath();\n    ctx.moveTo(chartLeft, chartTop);\n    ctx.lineTo(chartLeft, middleChartHeight);\n    ctx.moveTo(chartRight, chartTop);\n    ctx.lineTo(chartRight, chartBottom);\n    ctx.stroke();\n\n    // Draw X-axis ticks and labels\n    ctx.textAlign = \"center\";\n    ctx.textBaseline = \"top\";\n    const halfDx = dx / 2;\n    for (let i = 0; i < 2 ** qubitWidth; i++) {\n      const x = i * dx + chartLeft;\n      const binary = i.toString(2).padStart(qubitWidth, \"0\");\n      ctx.beginPath();\n      ctx.moveTo(x, middleChartHeight - 5);\n      ctx.lineTo(x, middleChartHeight + 5);\n      ctx.stroke();\n      ctx.fillText(binary, x + halfDx, middleChartHeight + 10);\n    }\n\n    // Draw left Y-axis ticks and labels\n    ctx.textAlign = \"right\";\n    ctx.textBaseline = \"middle\";\n    for (let i = 0; i <= 10; i++) {\n      const tickY = chartTop + (i / 10) * dy;\n      const label = ((10 - i) / 10).toFixed(1);\n      ctx.beginPath();\n      ctx.moveTo(chartLeft - 5, tickY);\n      ctx.lineTo(chartLeft, tickY);\n      ctx.stroke();\n      ctx.fillText(label, chartLeft - 8, tickY);\n    }\n\n    // Draw right Y-axis ticks and labels\n    ctx.textAlign = \"left\";\n    ctx.textBaseline = \"middle\";\n\n    // Draw 180° and -180° labels\n    ctx.beginPath();\n    ctx.moveTo(chartRight + 5, chartTop);\n    ctx.lineTo(chartRight, chartTop);\n    ctx.stroke();\n    ctx.fillText(\"180°\", chartRight + 8, chartTop);\n\n    ctx.beginPath();\n    ctx.moveTo(chartRight + 5, chartBottom);\n    ctx.lineTo(chartRight, chartBottom);\n    ctx.stroke();\n    ctx.fillText(\"-180°\", chartRight + 8, chartBottom);\n\n    // Draw other labels\n    function tickAtTheMiddle(topPos, bottomPos, topValue, bottomValue, steps, currentStep = 0) {\n      if (currentStep > steps) {\n        return;\n      }\n\n      const tickY = (topPos + bottomPos) / 2;\n      const value = ((topValue + bottomValue) / 2);\n      const label = `${value.toFixed(1)}°`;\n\n      ctx.textAlign = \"left\";\n      ctx.textBaseline = \"middle\";\n\n      ctx.beginPath();\n      ctx.moveTo(chartRight + 5, tickY);\n      ctx.lineTo(chartRight, tickY);\n      ctx.stroke();\n      ctx.fillText(label, chartRight + 8, tickY);\n\n      tickAtTheMiddle(topPos, tickY, topValue, value, steps, currentStep + 1);\n      tickAtTheMiddle(tickY, bottomPos, value, bottomValue, steps, currentStep + 1);\n    }\n\n    tickAtTheMiddle(chartTop, chartBottom, 180.0, -180.0, 3);\n  }\n\n  _drawAmpitudes(statevector) {\n    const ctx = this._ctx;\n    const {\n      _chartTop: chartTop,\n      _chartLeft: chartLeft,\n      _dx: dx,\n      _dy: dy\n    } = this;\n\n    // Draw amplitude bars\n    const prevFillStyle = ctx.fillStyle;\n    const prevStrokeStyle = ctx.strokeStyle;\n    ctx.fillStyle = \"#0077be\";\n    ctx.strokeStyle = \"#00072d\";\n    ctx.textAlign = \"center\";\n    ctx.textBaseline = \"middle\";\n    ctx.lineWidth = 1;\n    const amplitudes = statevector.bases;\n    const barWidth = dx * 0.8;\n    const halfDx = dx / 2;\n    const semiBar = barWidth / 2;\n    for (let i = 0; i < amplitudes.length; i += 2) {\n      const real = amplitudes[i];\n      const imag = amplitudes[i + 1];\n      const magnitude = Math.sqrt(real ** 2 + imag ** 2);\n      const x = (i / 2) * dx + chartLeft + halfDx - semiBar;\n      const barHeight = magnitude * dy;\n      const y = chartTop + dy - barHeight;\n      ctx.fillRect(x, y, barWidth, barHeight);\n      ctx.strokeRect(x, y, barWidth, barHeight);\n\n      ctx.beginPath();\n      ctx.fillText(magnitude.toFixed(3), x + semiBar, y - 20);\n    }\n    ctx.fillStyle = prevFillStyle;\n    ctx.strokeStyle = prevStrokeStyle;\n  }\n\n  _drawPhases(statevector) {\n    const ctx = this._ctx;\n    const {\n      _chartLeft: chartLeft,\n      _chartHeight: chartHeight,\n      _chartMiddle: chartMiddle,\n      _dx: dx\n    } = this;\n\n    // Draw phase points\n    const prevFillStyle = ctx.fillStyle;\n    ctx.fillStyle = \"limegreen\";\n    const amplitudes = statevector.bases;\n    const halfDx = dx / 2;\n    const radius = 3;\n    for (let i = 0; i < amplitudes.length; i += 2) {\n      const real = amplitudes[i];\n      const imag = amplitudes[i + 1];\n      const phase = Math.atan2(imag, real) / Math.PI * 180; // in degrees\n      const x = chartLeft + (i / 2) * dx + halfDx;\n      const y = chartMiddle - phase / 360 * chartHeight;\n      ctx.beginPath();\n      ctx.arc(x, y, radius, 0, 2 * Math.PI);\n      ctx.fill();\n      ctx.fillText(phase.toFixed(2), x, y - 10);\n    }\n    ctx.fillStyle = prevFillStyle;\n  }\n}\n\n\n//# sourceURL=webpack://SandBox/./src/plotter.js?");

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = __webpack_modules__;
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/get javascript chunk filename */
/******/ 	(() => {
/******/ 		// This function allow to reference async chunks
/******/ 		__webpack_require__.u = (chunkId) => {
/******/ 			// return url for filenames based on template
/******/ 			return "" + chunkId + ".index.js";
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/global */
/******/ 	(() => {
/******/ 		__webpack_require__.g = (function() {
/******/ 			if (typeof globalThis === 'object') return globalThis;
/******/ 			try {
/******/ 				return this || new Function('return this')();
/******/ 			} catch (e) {
/******/ 				if (typeof window === 'object') return window;
/******/ 			}
/******/ 		})();
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/publicPath */
/******/ 	(() => {
/******/ 		var scriptUrl;
/******/ 		if (__webpack_require__.g.importScripts) scriptUrl = __webpack_require__.g.location + "";
/******/ 		var document = __webpack_require__.g.document;
/******/ 		if (!scriptUrl && document) {
/******/ 			if (document.currentScript)
/******/ 				scriptUrl = document.currentScript.src
/******/ 			if (!scriptUrl) {
/******/ 				var scripts = document.getElementsByTagName("script");
/******/ 				if(scripts.length) scriptUrl = scripts[scripts.length - 1].src
/******/ 			}
/******/ 		}
/******/ 		// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration
/******/ 		// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.
/******/ 		if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
/******/ 		scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
/******/ 		__webpack_require__.p = scriptUrl;
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/jsonp chunk loading */
/******/ 	(() => {
/******/ 		__webpack_require__.b = document.baseURI || self.location.href;
/******/ 		
/******/ 		// object to store loaded and loading chunks
/******/ 		// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 		// [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
/******/ 		var installedChunks = {
/******/ 			"main": 0
/******/ 		};
/******/ 		
/******/ 		// no chunk on demand loading
/******/ 		
/******/ 		// no prefetching
/******/ 		
/******/ 		// no preloaded
/******/ 		
/******/ 		// no HMR
/******/ 		
/******/ 		// no HMR manifest
/******/ 		
/******/ 		// no on chunks loaded
/******/ 		
/******/ 		// no jsonp function
/******/ 	})();
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module can't be inlined because the eval devtool is used.
/******/ 	var __webpack_exports__ = __webpack_require__("./src/index.js");
/******/ 	var __webpack_export_target__ = (SandBox = typeof SandBox === "undefined" ? {} : SandBox);
/******/ 	for(var i in __webpack_exports__) __webpack_export_target__[i] = __webpack_exports__[i];
/******/ 	if(__webpack_exports__.__esModule) Object.defineProperty(__webpack_export_target__, "__esModule", { value: true });
/******/ 	
/******/ })()
;