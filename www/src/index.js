// Get references to HTML elements
const codeInput = document.getElementById("code-input");
const runButton = document.getElementById("run-button");
const outputContainer = document.getElementById("output-container");
const outputCanvas = document.getElementById("output-canvas");
const logOutput = document.getElementById("output-log");

// Set canvas resolution equal to container surface
outputCanvas.width = outputContainer.offsetWidth;
outputCanvas.height = outputContainer.offsetHeight;

// Get canvas context
const ctx = outputCanvas.getContext("2d");

// Initialize web worker for simulator
const worker = new Worker(new URL("./worker.js", import.meta.url));

// Add message listener to worker
worker.addEventListener("message", event => {
  const message = event.data;

  switch (message.type) {
    case "simulationComplete":
      // Draw the state vector
      drawStateVector(message.result.statevector);

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

function drawStateVector(statevector) {
  // Set padding
  const paddingLeft = 50;
  const paddingRight = 50;
  const paddingTop = 40;
  const paddingBottom = 50;

  // Clear canvas
  ctx.clearRect(0, 0, outputCanvas.width, outputCanvas.height);

  // Get number of qubits
  const numQubits = statevector.qubitWidth;

  // Draw axes
  drawAxes(numQubits, paddingLeft, paddingRight, paddingTop, paddingBottom);

  // Plot state vector
  drawAmplitudes(statevector, 40, paddingLeft, paddingRight, paddingTop, paddingBottom);
  drawPhases(statevector, 3, paddingLeft, paddingRight, paddingTop, paddingBottom);
}

function drawAxes(numQubits, paddingLeft, paddingRight, paddingTop, paddingBottom) {
  // Get canvas dimensions
  const canvasWidth = outputCanvas.width;
  const canvasHeight = outputCanvas.height;

  // Adjust chart dimensions for padding
  const chartWidth = canvasWidth - paddingLeft - paddingRight;
  const chartHeight = canvasHeight - paddingTop - paddingBottom;

  // Get unit vector lengths
  const dx = chartWidth / (2 ** numQubits);
  const dy = chartHeight / 2;

  // Draw X-axis
  ctx.strokeStyle = "#000";
  ctx.beginPath();
  ctx.moveTo(paddingLeft, canvasHeight - paddingBottom - dy);
  ctx.lineTo(canvasWidth - paddingRight, canvasHeight - paddingBottom - dy);
  ctx.stroke();

  // Draw Y-axes
  ctx.beginPath();
  ctx.moveTo(paddingLeft, paddingTop);
  ctx.lineTo(paddingLeft, dy + paddingTop);
  ctx.moveTo(canvasWidth - paddingRight, paddingTop);
  ctx.lineTo(canvasWidth - paddingRight, canvasHeight - paddingBottom);
  ctx.stroke();

  // Draw X-axis ticks and labels
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  const halfDx = dx / 2;
  for (let i = 0; i < 2 ** numQubits; i++) {
    const x = i * dx + paddingLeft;
    const binary = i.toString(2).padStart(numQubits, "0");
    ctx.beginPath();
    ctx.moveTo(x, canvasHeight - paddingBottom - dy - 5);
    ctx.lineTo(x, canvasHeight - paddingBottom - dy + 5);
    ctx.stroke();
    ctx.fillText(binary, x + halfDx, canvasHeight - paddingBottom - dy + 10);
  }

  // Draw left Y-axis ticks and labels
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  for (let i = 0; i <= 10; i++) {
    const tickY = paddingTop + dy * i / 10;
    const label = ((10 - i) / 10).toFixed(1);
    ctx.beginPath();
    ctx.moveTo(paddingLeft - 5, tickY);
    ctx.lineTo(paddingLeft, tickY);
    ctx.stroke();
    ctx.fillText(label, paddingLeft - 8, tickY);
  }

  // Draw right Y-axis ticks and labels
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";

  // Draw π and -π labels
  ctx.beginPath();
  ctx.moveTo(canvasWidth - paddingRight + 5, paddingTop);
  ctx.lineTo(canvasWidth - paddingRight, paddingTop);
  ctx.stroke();
  ctx.fillText("180", canvasWidth - paddingRight + 8, paddingTop);

  ctx.beginPath();
  ctx.moveTo(canvasWidth - paddingRight + 5, canvasHeight - paddingBottom);
  ctx.lineTo(canvasWidth - paddingRight, canvasHeight - paddingBottom);
  ctx.stroke();
  ctx.fillText("-180", canvasWidth - paddingRight + 8, canvasHeight - paddingBottom);

  // Draw other labels
  function tickAtTheMiddle(topPos, bottomPos, topValue, bottomValue, steps, currentStep = 0) {
    if (currentStep > steps) {
      return;
    }

    const tickY = (topPos + bottomPos) / 2;
    const value = ((topValue + bottomValue) / 2);
    const label = value.toFixed(1);

    ctx.textAlign = "left";
    ctx.textBaseline = "middle";

    ctx.beginPath();
    ctx.moveTo(canvasWidth - paddingRight + 5, tickY);
    ctx.lineTo(canvasWidth - paddingRight, tickY);
    ctx.stroke();
    ctx.fillText(label, canvasWidth - paddingRight + 8, tickY);

    tickAtTheMiddle(topPos, tickY, topValue, value, steps, currentStep + 1);
    tickAtTheMiddle(tickY, bottomPos, value, bottomValue, steps, currentStep + 1);
  }

  tickAtTheMiddle(paddingTop, canvasHeight - paddingBottom, 180.0, -180.0, 3);
}

function drawPhases(statevector, radius, paddingLeft, paddingRight, paddingTop, paddingBottom) {
  // Get canvas dimensions
  const canvasWidth = outputCanvas.width;
  const canvasHeight = outputCanvas.height;

  // Adjust chart dimensions for padding
  const chartWidth = canvasWidth - paddingLeft - paddingRight;
  const chartHeight = canvasHeight - paddingTop - paddingBottom;

  // Get number of qubits
  const numQubits = statevector.qubitWidth;

  // Get unit vector lengths
  const dx = chartWidth / (2 ** numQubits);
  const dy = chartHeight / 2;

  // Draw phase points
  const prevFillStyle = ctx.fillStyle;
  ctx.fillStyle = "limegreen";
  const amplitudes = statevector.bases;
  const halfDx = dx / 2;
  for (let i = 0; i < amplitudes.length; i += 2) {
    const real = amplitudes[i];
    const imag = amplitudes[i + 1];
    const phase = Math.atan2(imag, real) / Math.PI * 180; // in degrees
    const x = (i / 2) * dx + paddingLeft + halfDx;
    const y = (1 - phase / 360) * chartHeight + paddingTop - dy;
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, 2 * Math.PI);
    ctx.fill();
    ctx.fillText(phase.toFixed(2), x, y - 10);
  }
  ctx.fillStyle = prevFillStyle;
}

function drawAmplitudes(statevector, barWidth, paddingLeft, paddingRight, paddingTop, paddingBottom) {
  // Get canvas dimensions
  const canvasWidth = outputCanvas.width;
  const canvasHeight = outputCanvas.height;

  // Adjust chart dimensions for padding
  const chartWidth = canvasWidth - paddingLeft - paddingRight;
  const chartHeight = canvasHeight - paddingTop - paddingBottom;

  // Get number of qubits
  const numQubits = statevector.qubitWidth;

  // Get unit vector lengths
  const dx = chartWidth / (2 ** numQubits);
  const dy = chartHeight / 2;

  // Draw amplitude bars
  const prevFillStyle = ctx.fillStyle;
  const prevStrokeStyle = ctx.strokeStyle;
  ctx.fillStyle = "#0077be";
  ctx.strokeStyle = "#00072d";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.lineWidth = 1;
  const amplitudes = statevector.bases;
  const halfDx = dx / 2;
  const halfWidth = barWidth / 2;
  for (let i = 0; i < amplitudes.length; i += 2) {
    const real = amplitudes[i];
    const imag = amplitudes[i + 1];
    const magnitude = Math.sqrt(real ** 2 + imag ** 2);
    const x = (i / 2) * dx + paddingLeft + halfDx - halfWidth;
    const height = magnitude * dy;
    const y = paddingTop + dy - height;
    ctx.fillRect(x, y, barWidth, height);
    ctx.strokeRect(x, y, barWidth, height);

    ctx.beginPath();
    ctx.fillText(magnitude.toFixed(3), x + halfWidth, y - 20);
  }
  ctx.fillStyle = prevFillStyle;
  ctx.strokeStyle = prevStrokeStyle;
}