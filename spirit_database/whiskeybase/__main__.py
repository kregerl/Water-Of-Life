import re
import json
import requests
import xml.etree.ElementTree as ET
from bs4 import BeautifulSoup
from typing import List, Dict
from dataclasses import dataclass

def get_xml_tree(url: str) -> ET:
    response = requests.get(url)
    return ET.fromstring(response.text)


def extract_whiskey_urls():
    urls = []
    root = get_xml_tree("https://www.whiskybase.com/sitemaps/sitemaps.xml")
    for child in root:
        whiskey_url = child[0].text
        if re.search("whiskies-[0-9]+", whiskey_url) is not None:
            urls.append(whiskey_url)
    return urls


@dataclass
class WhiskeyData:
    title: str
    data_url: str 
    image_url: str


def extract_whiskey_locations_and_images(url: str) -> List[WhiskeyData]:
    data = []
    root = get_xml_tree(url)
    for child in root:
        url = child.find(path="{http://www.sitemaps.org/schemas/sitemap/0.9}loc").text
        image_url = getattr(child.find(path="{http://www.google.com/schemas/sitemap-image/1.1}image/{http://www.google.com/schemas/sitemap-image/1.1}loc"), "text", None)
        title = getattr(child.find(path="{http://www.google.com/schemas/sitemap-image/1.1}image/{http://www.google.com/schemas/sitemap-image/1.1}caption"), "text", None)
        data.append(WhiskeyData(title, url, image_url))
    return data


def table_as_dict(parser: BeautifulSoup, details) -> Dict[str, str]:
    keys = []
    values = []
    for dt in parser.findAll("dt"):
        keys.append(dt.text.strip())
    for dd in parser.findAll("dd"):
        values.append(dd.text.strip()) 
    return dict(zip(keys, values))


def extract_whiskey_data(whiskey_data: WhiskeyData) -> Dict[str, str]:
    headers = {
        "Host": "www.whiskybase.com",
        "User-Agent": "Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    }
    response = requests.get(whiskey_data.data_url, headers=headers)
    parser = BeautifulSoup(response.content, "html.parser")
    details = parser.select("#whisky-details dl")
    table = table_as_dict(parser, details)
    table["Name"] = whiskey_data.title
    return table

def save_whiskey_data(whiskey_urls: List[str]) -> List[Dict]:
    results  = []
    for whiskey_url in whiskey_urls:
        all_attributes = extract_whiskey_data(whiskey_url)
        results.append({
            "Name": all_attributes.get("Name"),
            "Category": all_attributes.get("Category", None),
            "Distillery": all_attributes.get("Distillery", None),
            "Stated Age": all_attributes.get("Stated Age", None),
            "Cask Type": all_attributes.get("Cask Type", None),
            "Strength": all_attributes.get("Strength", None),
            "Size": all_attributes.get("Size", None),
            "Barcode": all_attributes.get("Barcode", None)
        })
    return results

if __name__ == "__main__":
    urls = extract_whiskey_urls()
    whiskey_urls = extract_whiskey_locations_and_images(urls[0])
    results = save_whiskey_data(whiskey_urls)
    with open("test.json", "w+") as f:
        json.dump(results, f, indent=4)
