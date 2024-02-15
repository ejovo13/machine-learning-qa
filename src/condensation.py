"""Condense different json streams into a single json file."""

import os.path as os_path
import os
import json
from .extraction.const import DEFAULT_MULTISTREAM_JSON_DIR
import polars as pl

DEFAULT_CONDENSED_JSON_OUT = "/home/ejovo/MAIN/S9/machine_learning/project/data/json/condensed.json"

def condense_multistream_json(
    json_dir: str = DEFAULT_MULTISTREAM_JSON_DIR,
    output_file: str = DEFAULT_CONDENSED_JSON_OUT
):
    """Read all of the files in `json_dir` and condense their content into a single file."""

    condensed_dict = {}

    for (idx, file) in enumerate(os.listdir(json_dir), 1):

        print("[{:03}]".format(idx))

        if file[-5:] != ".json": continue
        full_filename = os_path.join(json_dir, file)

        with open(full_filename) as stream:
            single_stream: dict = json.loads(stream.read())

            for key, value in single_stream.items():
                if value["text"] == "": continue
                condensed_dict[key] = value

    with open(output_file, "w") as out_json:
        out_json.write(json.dumps(condensed_dict, indent=4))

def inspect_condensed(json_filename: str = DEFAULT_CONDENSED_JSON_OUT):
    """Print some information about our condensed dump."""
    articles = get_articles(json_filename)
    print("Num articles:")
    print(len(articles.keys()))

    count_empty = 0

    for article_title, metadata in articles.items():
        if metadata["text"] == "": count_empty += 1

    print("Num empty:")
    print(count_empty)

def get_articles(json_filename: str = DEFAULT_CONDENSED_JSON_OUT) -> dict[str, dict]:

    with open(json_filename) as json_file:
        articles: dict = json.loads(json_file.read())

    return articles

def get_articles_df(json_filename: str = DEFAULT_CONDENSED_JSON_OUT) -> pl.DataFrame:
    articles = get_articles(json_filename)
    titles = list(articles.keys())
    text = [value["text"] for value in articles.values()]
    lengths = [len(t) for t in text]
    ids = [value["id"] for value in articles.values()]
    hashes = [value["text_hash"] for value in articles.values()]

    return pl.DataFrame(dict(
        id=ids,
        title=titles,
        text=text,
        length=lengths,
        hash=hashes
    ))