/**
 * @typedef {Object} Barrel
 * @property {string} address
 * @property {number} index_size_bytes
 * @property {boolean} online
 */

/**
 * @typedef {Object} TopSearch
 * @property {string} search - Search
 * @property {number} count - count of search
 */

/**
 * @typedef {Object} WsResponse
 * @property {number} avg_response_time_ms - Average response time in miliseconds
 * @property {[Barrel]} barrels - barrells statuses
 * @property {[string]} queue - Queue of urls
 * @property {[TopSearch]} top10_searches - Top 10 Searches
 */

class DashboardElements {
  /**
   * @constructor
   * @param {boolean} enableUIUpdate - Whether to update ui
   */
  constructor(enableUIUpdate) {
    /** @type {boolean} */
    this.enableUIUpdate = enableUIUpdate;

    /** @type {number} */
    this.samples = 0;

    /** @type {HTMLElement} */
    this.wSocket = document.getElementById('td-ws');
    console.assert(this.wSocket !== null);

    /** @type {HTMLElement} */
    this.avgResponseTime = document.getElementById('td-avg-response-time');
    console.assert(this.avgResponseTime !== null);

    /** @type {HTMLUListElement} */
    this.barrelsList = document.getElementById('ul-barrels');
    console.assert(this.barrelsList !== null);

    /** @type {HTMLUListElement} */
    this.queueList = document.getElementById('ol-queue');
    console.assert(this.queueList !== null);

    /** @type {HTMLOListElement} */
    this.topSearchesList = document.getElementById('ol-search');
    console.assert(this.topSearchesList !== null);

    const wsSocket = new WebSocket('/ws');
    wsSocket.onopen = event => {
      console.log(`WebSocket connection opened ${event.target.url}`);

      wsSocket.send(JSON.stringify({
        action: "subscribe",
        topic: "status",
      }));

      this.updateWebSocketStatus(true);
    };

    wsSocket.onmessage = event => {
      try {
        this.updateUI(JSON.parse(event.data));
      } catch (err) {
        console.error("Error parsing message:", err);
      }
    };

    wsSocket.onerror = error => {
      console.error('WebSocket error:', error);

      this.updateWebSocketStatus(false);
    };

    /** @type {WebSocket} */
    this.wsSocket = wsSocket;
    window.wsSocket = this.wsSocket;
  }

  /**
   * Updates the WebSocket connection status in the UI.
   * @param {boolean} isConnected - Indicates whether the WebSocket is connected or not.
   */
  updateWebSocketStatus(isConnected) {
    this.wSocket.className = isConnected ? 'highlight-yes' : 'highlight-no';
    this.wSocket.innerHTML = isConnected ? 'Yes' : 'No';
  }

  /**
   * Updates UI
   * @param {WsResponse} data 
  */
  updateUI(data) {
    console.log(data);

    if (!this.updateUI) {
      return;
    }

    const {
      avg_response_time_ms,
      barrels,
      queue,
      top10_searches
    } = data;

    console.assert(avg_response_time_ms !== undefined);
    console.assert(barrels !== undefined);
    console.assert(queue !== undefined);
    console.assert(top10_searches !== undefined);

    // Update Average response time
    this.samples += 1;
    this.avgResponseTime.innerHTML = `${avg_response_time_ms.toFixed(3)} (${this.samples} samples)`;

    // Update barrels list
    this.barrelsList.innerHTML = barrels.length == 0 ? 'No barrels' :
      barrels.map(({ address, online }) => {
        const onlineS = online ? 'Online' : 'Offline';
        const className = online ? 'highlight-yes' : 'highlight-no';

        return `<li class="${className}">${address}: ${onlineS}</li>`;
      }).join('\n');

    // Update queue list
    this.queueList.innerHTML = queue.length == 0 ? 'Queue is empty' :
      queue.map(url => `<li>${url}</li>`).join('\n');

    // Update top 10 searches list
    this.topSearchesList.innerHTML = top10_searches.length == 0 ? 'No searches available' :
      top10_searches.map(({ search, count }) => `<li>(${count}) ${search}</li>`).join('\n');
  }
}

const elements = new DashboardElements(true);
console.log(elements);
