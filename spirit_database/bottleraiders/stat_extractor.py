import json
import html
import requests
from typing import Dict, Tuple
from bs4 import BeautifulSoup

def get_stats(url: str) -> Tuple[str | None, Dict[str, str]]:
    response = requests.get(url)
    parser = BeautifulSoup(response.content, "html.parser")
    image = None
    try:
        image = parser.select(".o-spirit-image")[0]["src"]
    except IndexError:
        image = None
    stat_list = parser.select(".o-spirit-stat-list")

    if stat_list is None or len(stat_list) == 0:
        return (None, {})

    keys = []
    values = []
    for list_element in stat_list[0].findAll("li"):
        keys.append(list_element.select(".o-spirit-stat-key")[0].text.strip().replace(":", ""))
        values.append(list_element.select(".o-spirit-stat-value")[0].text.strip())
    return (image, dict(zip(keys, values)))



if __name__ == "__main__":
    with open("whiskey.json", "r") as f:
        contents = json.load(f)

        results = []
        total = contents["size"]
        index = 0
        for element in contents["whiskey"]:
            print(f"Getting whiskey {index}/{total}...")
            image, stats = get_stats(element["permalink"])
            result = {
                "Title": html.unescape(element["title"]),
                "Type": element["type"],
                **stats,
            }

            if image:
                result["Image"] = image
            results.append(result)
            index += 1

        with open("whiskey_stats.json", "w+") as rf:
            json.dump(results, fp=rf)