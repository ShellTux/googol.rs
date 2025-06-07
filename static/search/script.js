"use-strict";

const elements = {
  dummyCheckbox: document.getElementById('dummy-checkbox'),
  webServerAddressInput: document.getElementById('webserver-address-input'),
  hackerNewsButton: document.getElementById('hackernews-button'),
  resultsCountDiv: document.getElementById('results-count'),
  resultsDiv: document.getElementById('results'),
  searchButton: document.getElementById('search-button'),
  searchInput: document.getElementById('search-input'),
};

console.debug('elements', elements);

/**
 * @typedef {Object} Page
 * @property {string} href.
 * @property {string=} title.
 * @property {string=} summary.
 * @property {string=} icon.
 */

/**
 * Get results
 * @param {string[]} words - Array of string search terms.
 * @returns {Page[]}
 */
const getResults = async words => {
  console.debug('words', words);

  const dummy = elements.dummyCheckbox.checked;

  if (dummy) {
    return [
      {
        href: 'https://google.com',
        title: 'Google',
        summary: 'Google search engine'
      },
      {
        href: 'https://facebook.com',
        title: 'Facebook',
        summary: 'Facebook social media'
      },
      {
        href: 'https://youtube.com',
        title: 'Youtube',
      },
      {
        href: 'https://en.wikipedia.org/wiki/Lion',
        title: 'Lion - Wikipedia',
        icon: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABwAAAAcCAAAAABXZoBIAAAAnklEQVR4AeTNIQiDQABG4b+u17X1aF6PK3YEO9iMJqPVau82y4FgMezS0oVLhqsHtrcqeqzDXv3CEz/6L4yTtZM3dnHmPTtjzXZAXKYVo4agkU2GI2Lloc6JDez1+flswMu1EQZ3xlE7lK8eKDkjtwE+crBMV+wesKmCiisGGepZIfQJpMj9SNb2MYWrChjVkULuCyCfRvsdmBieyQQAsoDk/9ryhFMAAAAASUVORK5CYII=',
      },
    ];
  }

  const webServerAddress = elements.webServerAddressInput.value.trim();
  if (!webServerAddress) {
    alert("Insert a valid WebServer address!");
    return [];
  }

  /** @type {Page[]} */
  const results = await fetch(`http://${webServerAddress}/search?words=${words.concat(',')}`)
    .then(response => {
      if (!response.ok) {
        // Handle HTTP errors
        throw new Error(`Network response was not ok: ${response.statusText}`);
      }

      return response;
    })
    .then(response => response.json())
    // .then(data => { console.log('data', data); return data; })
    .then(data => data.map(page => {
      const { title, href, summary, icon } = page;

      /** @type{Page} */
      const pageO = { title, href, summary, icon };

      return pageO;
    }))
    .then(data => { console.log('data', data); return data; })
    .catch(error => console.error('There was a problem with the fetch operation: ', error));

  return results;
}

elements.searchButton.addEventListener('click', async (event) => {
  console.debug('event', event);

  /** @type {string} */
  const query = elements.searchInput.value.trim().toLowerCase();
  console.debug('query', query);

  const searchTerms = query.split(' ').filter(word => word.length > 0);
  console.log('searchTerms', searchTerms);

  const results = await getResults(searchTerms);
  console.debug('results', results);

  elements.resultsCountDiv.innerHTML = `${results.length} Results found.`;

  elements.resultsDiv.innerHTML = '';

  results.map(page => {
    console.debug('page', page);

    let { title, href, summary, icon } = page;

    const resultDiv = document.createElement('div');
    resultDiv.className = 'result';

    const linkA = document.createElement('a');
    linkA.href = href;
    linkA.textContent = title.length > 0 ? title : href;
    linkA.target = '_blank'; // optional: open in new tab
    linkA.style.display = 'block'; // optional: display each link on its own line

    if (icon) {
      const img = document.createElement('img');
      img.src = icon;
      img.alt = 'Icon';
      img.style.width = '20px';
      img.style.height = '20px';
      img.style.marginRight = '8px';
      linkA.appendChild(img);
      // resultDiv.appendChild(img);
    }

    resultDiv.appendChild(linkA);

    if (summary && summary.length > 0) {
      const summaryPara = document.createElement('p');
      summaryPara.textContent = summary;
      resultDiv.appendChild(summaryPara);
    } else {
      const summarizeButton = document.createElement('button');
      summarizeButton.id = 'summarize-button';
      summarizeButton.innerHTML = 'AI Summarize';
      summarizeButton.style.margin = '0 0 10px 0';

      summarizeButton.addEventListener('click', _ => {
        console.log(`Summarizing ${page.href} ...`);
        alert('Summarize feature coming soon!');
      });

      resultDiv.appendChild(summarizeButton);
    }

    return resultDiv;
  }).forEach(resultDiv => {
    elements.resultsDiv.appendChild(resultDiv);
  });
});
