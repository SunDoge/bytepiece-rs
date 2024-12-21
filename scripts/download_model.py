# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "requests",
# ]
# ///


import argparse
import os
import requests
from pathlib import Path
from zipfile import ZipFile

MODELS = [
    "bytepiece_80k",
    "bytepiece_160k",
]
BASE_URL = "https://github.com/bojone/bytepiece/raw/main/models/"
DEST_DIR = "models"


parser = argparse.ArgumentParser()
parser.add_argument("-m", "--model", choices=MODELS, default=MODELS[0])

args = parser.parse_args()

dest_dir = Path(DEST_DIR)
dest_dir.mkdir(exist_ok=True)

model_name: str = args.model
model_filename = model_name + ".zip"
model_url = BASE_URL + model_filename
model_file_path = dest_dir / model_filename

print(model_filename)
print(model_url)

if not model_file_path.exists():
    resp = requests.get(model_url)
    with model_file_path.open("wb") as f:
        f.write(resp.content)
else:
    print(f"{model_file_path} exists")

with ZipFile(model_file_path) as zip_file:
    zip_file.extractall(dest_dir)
