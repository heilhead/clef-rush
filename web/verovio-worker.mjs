import createVerovioModule from "./verovio/dist/verovio-module.mjs";
import { VerovioToolkit } from "./verovio/dist/verovio.mjs";
import * as config from "./config.mjs";

const tk = new VerovioToolkit(await createVerovioModule());

addEventListener("message", (message) => {
  // console.log("processing request", message.data);

  const { type, data } = message.data;
  let result = { response: null };

  try {
    switch (type) {
      case config.RPC_PING:
        break;
      case config.RPC_SET_OPTIONS:
        setOptions(data);
        break;
      case config.RPC_GET_OPTIONS:
        result.response = getOptions();
        break;
      case config.RPC_RESET_OPTIONS:
        resetOptions();
        break;
      case config.RPC_CONVERT_TO_SVG:
        result.response = convertToSVG(data);
        break;
      default:
        result.error = `unknown message type: ${type}`;
    }
  } catch (err) {
    result.error = err.toString();
  } finally {
    message.ports[0].postMessage(result);
  }
});

function setOptions(opts) {
  tk.setOptions(JSON.parse(opts));
}

function getOptions() {
  return JSON.stringify(tk.getOptions());
}

function resetOptions() {
  return tk.resetOptions();
}

function convertToSVG(data) {
  tk.loadData(data);
  return tk.renderToSVG(1);
}

postMessage("ready");

console.log("worker ready");
