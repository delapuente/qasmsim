const $ = (selector) => document.querySelector(selector);
const bgSimulator = new Worker(new URL('./worker.js', import.meta.url));

bgSimulator.onmessage = (evt) => {
  if (evt.data === 'ready') {
    $('#run').disabled = false;
  }
  if (evt.data.type === 'result') {
    showTimes(evt.data.result.times);
  }
};

function showTimes(result) {
  $('#output').innerHTML += Object
    .entries(result)
    .map(([key, value]) => `${key}: ${value.duration} ms`)
    .join('\n') + '\n\n';
}

export function run() {
  const source = $('#source').value;
  bgSimulator.postMessage(source);
}