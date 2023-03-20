// Load QASMSim module asynchronously
import("qasmsim").then(qasmsim => {
  // Add message listener to worker
  self.addEventListener("message", event => {
    const message = event.data;

    switch (message.type) {
      case "simulate":
        try {
          // Parse and simulate OpenQASM code
          const result = qasmsim.run(message.code);

          // Serialize performance measures to JSON-serializable format
          const times = {};
          Object.keys(result.times).forEach(key => {
            times[key] = result.times[key].toJSON();
          });

          // Send result object back to main thread
          self.postMessage({ type: "simulationComplete", result: { ...result, times } });
        } catch (error) {
          // Send error message back to main thread
          self.postMessage({ type: "simulationError", error: error.message });
        }
        break;

      default:
        console.warn("Unknown message type:", message.type);
        break;
    }
  });
});
