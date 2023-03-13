import('qasmsim').then((qasmsim) => {
  onmessage = (evt) => {
    console.log(`worker received: ${evt.data}`);
    const result = qasmsim.run(evt.data);
    console.log(result);
  };
  onerror = (evt) => {
    console.log(evt, 'worker error');
  };
  postMessage('ready');
});