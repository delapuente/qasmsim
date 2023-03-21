import { StatevectorPlotter } from "./plotter";

// Get references to HTML elements
const codeInput = document.getElementById("code-input");
const runButton = document.getElementById("run-button");
const outputContainer = document.getElementById("output-container");
const outputCanvas = document.getElementById("output-canvas");
const logOutput = document.getElementById("output-log");

// Set canvas resolution equal to container surface
outputCanvas.width = outputContainer.offsetWidth;
outputCanvas.height = outputContainer.offsetHeight;

// Statevector plotter
const plotter = new StatevectorPlotter(outputCanvas);

// Initialize web worker for simulator
const worker = new Worker(new URL("./worker.js", import.meta.url));

// Add message listener to worker
worker.addEventListener("message", event => {
  const message = event.data;

  switch (message.type) {
    case "simulationComplete":
      // Draw the state vector
      plotter.plot(message.result.statevector);

      // Display times
      const times = Object.entries(message.result.times).map(
        ([key, value]) => `${key}: ${value.duration} ms`
      ).join("\n");
      logOutput.innerText = times;

      // Re-enable the run button
      runButton.removeAttribute("disabled");
      break;

    case "simulationError":
      console.error("Simulation error:", message.error);

      // Display error message
      logOutput.innerText = message.error;

      // Re-enable the run button
      runButton.removeAttribute("disabled");
      break;

    default:
      console.warn("Unknown message type:", message.type);
      break;
  }
});

// Add event listener to run button
runButton.addEventListener("click", () => {
  const code = codeInput.value;

  // Disable the run button
  runButton.setAttribute("disabled", true);

  // Send OpenQASM code to web worker for simulation
  worker.postMessage({ type: "simulate", code: code });
});
