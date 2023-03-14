import('qasmsim').then((qasmsim) => {
  onmessage = (evt) => {
    console.log(`worker received: ${evt.data}`);
    const result = qasmsim.run(evt.data);
    const transferable = makeTransferable(result);
    postMessage({ type: 'result', result: transferable });
  };

  onerror = (evt) => {
    console.log(evt, 'worker error');
  };

  postMessage('ready');

  function makeTransferable(result) {
    const transferable = {
      memory: result.memory,
      statevector: result.statevector,
      probabilities: result.probabilities,
      times: Object.entries(result.times).reduce((obj, [key, value]) => {
        obj[key] = value.toJSON();
        return obj;
      }, {}),
    };
    return transferable;
  }
});