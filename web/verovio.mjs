import * as config from "./config.mjs";

const worker = new Worker("./web/verovio-worker.mjs", {
  type: "module",
});

const initPromise = new Promise((resolve) => {
  worker.onmessage = function ({ data }) {
    if (data !== "ready") {
      console.error("invalid init message", data);
      return;
    }

    worker.onmessage = null;
    console.log("worker initialized");
    resolve();
  };
});

function sendRequest(type, data) {
  const channel = new MessageChannel();

  worker.postMessage({ type, data }, [channel.port2]);

  return new Promise((resolve, reject) => {
    channel.port1.onmessage = function (message) {
      const data = message.data;

      if (data.error) {
        reject(data.error);
      } else {
        resolve(data.response);
      }
    };
  });
}

export class Verovio {
  static init() {
    return initPromise;
  }

  static ping() {
    return sendRequest(config.RPC_PING, null);
  }

  static getOptions() {
    return sendRequest(config.RPC_GET_OPTIONS, null);
  }

  static setOptions(options) {
    return sendRequest(config.RPC_SET_OPTIONS, options);
  }

  static resetOptions() {
    return sendRequest(config.RPC_RESET_OPTIONS, null);
  }

  static convertToSVG(meiXml) {
    return sendRequest(config.RPC_CONVERT_TO_SVG, meiXml);
  }
}

window.Verovio = Verovio;
