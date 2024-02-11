"""Scripts to extract the information from wikipedia's dumps."""

# Our first attempt is to decompress the very first stream in our file.

import bz2
from icecream import ic
import json
# import pprint
# import xmltodict
import xml.etree.ElementTree as ET

DEFAULT_DUMP_FILE = "/home/ejovo/MAIN/S9/machine_learning/project/data/dump/simplewiki-20240201-pages-articles-multistream.xml.bz2"

def open_dump(filename: str = DEFAULT_DUMP_FILE):

    file = bz2.open(filename, 'rb')
    print(f"Opened new file: {file}")
    print(f"| seekable: {file.seekable()}")
    file.close()

def read_dump_stream(start_index: int, end_index: int, filename: str = DEFAULT_DUMP_FILE) -> bytes:
    """Try and decompress a dump file starting from bytes `start_index` to `end_index

    Parameters
    ----------
    start_index : int
        the file position (in bytes) where to start reading from.
    end_index : int
        the file position (exclusive; in bytes) where to stop reading.
    """
    num_bytes_to_read = end_index - start_index

    file = open(filename, 'rb')
    file.seek(start_index)
    stream_bytes = file.read1(num_bytes_to_read)

    return bz2.decompress(stream_bytes)

def open_index(filename: str) -> list[int]:
    """Return a list of indexes that designate the different multistreams in our data dump."""
    indices = set()
    line_count = 0

    with open(filename, 'r') as file:

        for line in file:
            line_count += 1
            index = line.split(':')[0]
            indices.add(int(index))

    print(f"Read: {line_count} lines")
    return sorted(list(indices))

def ms_xml_to_dict(filename: str) -> dict[str, str]:
    """Rip through an XML file and extract the articles in a dictionary containing the article name and the article's text."""

    # First thing to do is load up the file
    tree = ET.parse(filename)
    root = tree.getroot()

    out_dict: dict[str, str] = {}

    for page in root:
        # Now we want to get the article's name and it's raw text
        title_el = page.find("title")
        revision_el = page.find("revision")

        article_title = str(title_el.text)
        article_text = str(revision_el.find("text").text)
        # Set value in our dictionary
        out_dict[article_title] = article_text

    return out_dict

def process_article_text(article_text: str) -> str:
    """Strip unnecessary characters from our article text."""