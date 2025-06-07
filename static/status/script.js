"use-strict";

const elements = {
  avgResponseTimeSpan: document.getElementById('avg-response-time'),
  barrelsDiv: document.getElementById('barrels'),
  connectButton: document.getElementById('connect-button'),
  connectP: document.getElementById('connect-p'),
  queueUl: document.getElementById('queue'),
  top10Ol: document.getElementById('top10'),
  webServerAddressInput: document.getElementById('web-server-address-input'),
};

console.debug(elements);

/** @type{WebSocket} */
let socket;

/**
 * Connects to the WebSocket server at the specified URL.
 * @param {string} url - The WebSocket URL to connect to.
 */
const createSocket = (url) => {
  console.debug(`url = ${url}`);

  const socket = new WebSocket(url);

  socket.onopen = () => {
    console.log(`WebSocket connection opened ${url}`);

    socket.send(JSON.stringify({
      action: "subscribe",
      topic: "status",
    }));
  };

  socket.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      console.log(data);

      // Update average response time
      if (data.avg_response_time_ms !== undefined) {
        elements.avgResponseTimeSpan.textContent = data.avg_response_time_ms.toFixed(3);
      }

      // Update barrels
      if (Array.isArray(data.barrels)) {
        // Clear existing barrels
        elements.barrelsDiv.innerHTML = "";
        data.barrels.forEach(barrel => {
          const online = barrel.online ? "Online" : "Offline";
          const div = document.createElement("div");

          div.className = "barrel " + online.toLowerCase();
          div.innerHTML = `
            <strong>Address:</strong> ${barrel.address}<br/>
            <strong>Status:</strong> ${online}
          `;

          elements.barrelsDiv.appendChild(div);
        });
      }

      // Update queue
      if (Array.isArray(data.queue)) {
        elements.queueUl.innerHTML = "";
        data.queue.forEach(item => {
          const li = document.createElement("li");
          li.textContent = item;
          elements.queueUl.appendChild(li);
        });
      }

      // Update top10_searches
      if (Array.isArray(data.top10_searches)) {
        elements.top10Ol.innerHTML = "";
        data.top10_searches.forEach(search => {
          const li = document.createElement("li");
          li.textContent = search;
          elements.top10Ol.appendChild(li);
        });
      }
    } catch (err) {
      console.error("Error parsing message:", err);
    }
  };


  // Optional: handle errors
  socket.onerror = (error) => {
    console.error('WebSocket error:', error);
    elements.connectP.innerHTML = "WebSocket not connected";
  };

  return socket;
};

elements.connectButton.addEventListener('click', _ => {
  elements.connectP.innerHTML = "WebSocket not connected";

  const webServerAddress = elements.webServerAddressInput.value.trim();
  if (!webServerAddress) {
    alert("Insert valid WebServer Address!");
    return;
  }

  const webSocketUrl = `http://${webServerAddress}/ws`;
  socket = createSocket(webSocketUrl);
  console.debug(socket);

  elements.connectP.innerHTML = `WebSocket connected: ${webSocketUrl}`;
});
