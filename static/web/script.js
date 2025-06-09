"use-strict";

const elements = {
  indexButton: document.getElementById('index-button'),
  indexInput: document.getElementById('index-input'),
  webServerAddressInput: document.getElementById('webserver-address-input'),
};

console.debug(elements);

const urlRegex = /https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)/;

function isValidUrl(url) {
  return urlRegex.test(url);
}

elements.indexButton.addEventListener('click', () => {
  const url = elements.indexInput.value.trim();

  if (!url) {
    alert('Please enter a URL to index.');
    return;
  }

  if (!isValidUrl(url)) {
    alert(`Invalid url: ${url}`)
    return;
  }

  const webServerAddress = elements.webServerAddressInput.value.trim();
  if (!webServerAddress) {
    alert("Please enter a valid WebServer Address. e.g.: 127.0.0.1:8080");
    return;
  }

  fetch(`http://${webServerAddress}/enqueue`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ url })
  })
    .then(response => response.json()) // or response.json() if expecting JSON
    .then(console.debug)
    // .then(data => {
    //   alert('Request successful: ' + data);
    // })
    .catch(error => {
      alert('Error: ' + error);
    });
});
