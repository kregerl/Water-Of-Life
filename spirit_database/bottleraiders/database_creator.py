import re
import json
import uuid
import sqlite3
import requests

if __name__ == "__main__":
    con = sqlite3.connect("whiskey.db")
    cur = con.cursor()

    cur.execute("CREATE TABLE whiskey(title TEXT NOT NULL, type TEXT NOT NULL, distiller TEXT NOT NULL, bottler TEXT NOT NULL, abv INTEGER NOT NULL, age TEXT NOT NULL, image_uuid TEXT NOT NULL)")
    image_uuids = {}
    with open("whiskey_stats.json", "r") as f:
        contents = json.load(f)
        data = []
        for entry in contents:
            image = entry.get("Image")
            distiller = entry.get("Distiller")
            bottler = entry.get("Bottler")
            age = entry.get("Age")
            abv = entry.get("ABV")

            if image is None or distiller is None or bottler is None or age is None or abv is None:
                continue
            
            image_uuid = str(uuid.uuid4())
            image_uuids[image_uuid] = image

            print(abv,re.findall(r"[-+]?(?:\d*\.*\d+)", abv))
            abv = re.findall("[-+]?(?:\d*\.*\d+)", abv)[0]
            data.append((entry["Title"], entry["Type"], distiller, bottler, abv, age, image_uuid))
        cur.executemany("INSERT INTO whiskey VALUES (?, ?, ?, ?, ?, ?, ?)", data)
        con.commit()
    
    total = len(image_uuids)
    index = 0
    for key, value in image_uuids.items():
        print(f"Downloading image {index}/{total}")
        response = requests.get(value, stream=True)
        if response.status_code == 200:
            with open(f"../../spirit_images/{key}.jpg", "wb+") as f:
                for chunk in response:
                    f.write(chunk)
        index += 1

