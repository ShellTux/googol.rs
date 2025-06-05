"use-strict";

const elements = {
  indexButton: document.getElementById('index-button'),
  indexInput: document.getElementById('index-input'),
};

console.debug(elements);

function isValidUrl(url) {
  // TODO
  return true;
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

  // TODO: Url
  fetch('http://127.0.0.1:8080/enqueue', {
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
