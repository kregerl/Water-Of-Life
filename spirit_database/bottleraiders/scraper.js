async function getSpirits(variety) {
  let page = 1;
  let maxPages = -1;

  let spirits = [];
  do {
    let response = await fetch("https://bottleraiders.com/wp-admin/admin-ajax.php", {
      "headers": {
        "accept": "*/*",
        "accept-language": "en-US,en;q=0.8",
        "cache-control": "no-cache",
        "content-type": "multipart/form-data; boundary=----WebKitFormBoundary5wjqdfXf6iiLzOhy",
        "pragma": "no-cache",
        "sec-ch-ua": "\"Brave\";v=\"123\", \"Not:A-Brand\";v=\"8\", \"Chromium\";v=\"123\"",
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": "\"Linux\"",
        "sec-fetch-dest": "empty",
        "sec-fetch-mode": "cors",
        "sec-fetch-site": "same-origin",
        "sec-gpc": "1",
        "Referer": "https://bottleraiders.com/archive/?variety=" + variety,
        "Referrer-Policy": "strict-origin-when-cross-origin"
      },
      "body": "------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"searchTerm\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"action\"\r\n\r\nam-ajax-spirit-reviews\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"_ajax_nonce\"\r\n\r\n0d9210ccac\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"min\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"max\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"page\"\r\n\r\n" + page + "\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"firstChar\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"variety\"\r\n\r\n" + variety + "\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"type\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"orderby\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"compactColumn\"\r\n\r\nrating\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"order\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"price\"\r\n\r\n\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy\r\nContent-Disposition: form-data; name=\"isCompact\"\r\n\r\n0\r\n------WebKitFormBoundary5wjqdfXf6iiLzOhy--\r\n",
      "method": "POST"
    });
    let json = await response.json();
    maxPages = Math.max(maxPages, json["pagination"]["totalPages"]);

    spirits.push(json["spirits"])

    page += 1;
  } while (page <= maxPages)
  return spirits;
}

async function saveAllSpirits(varieties) {
  for (let variety of varieties) {
    let results = await getSpirits(variety);
    require("fs").writeFile(`${variety}.json`, JSON.stringify(results.flat()), (error) => {
      if (error) throw error;
    });
  }
}

varieties = ["whiskey", "rum", "agave", "gin", "vodka", "brandy", "liqueur"];

saveAllSpirits(varieties);