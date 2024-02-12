"""Extract relevant information from the entire our xml file."""

import os
import json
import os.path as os_path
from xml.etree import ElementTree as ET
from .bz2_extraction import open_index
from .const import *

from wikicleaner import clean_article_text


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


def process_multistream_xml(
    xml_directory: str = DEFAULT_MULTISTREAM_XML_DIR,
    json_directory: str = DEFAULT_MULTISTREAM_JSON_DIR,
    index_file: str = DEFAULT_INDEX_FILE,
):
    """Process the xml files in `xml_directory`, in serial, to extract article information in json format."""

    entries, _ = open_index(index_file)

    # Let's create a dictionary from our entries with article => id
    article_id_dict = {entry.article_title: entry.article_id for entry in entries}

    mk_filename = lambda stream_index: os_path.join(
        json_directory, "{:04}.json".format(stream_index)
    )

    for stream_id, xml_file in enumerate(os.listdir(xml_directory)[181:182], 1):

        filename_full = os_path.join(xml_directory, xml_file)
        tree = ET.parse(filename_full)
        root = tree.getroot()

        out_dict = {}
        print(f"[{stream_id:03}]")

        for page in root:
            # Now we want to get the article's name and it's raw text
            title_el = page.find("title")
            revision_el = page.find("revision")

            article_title = title_el.text

            try:
                input_text = revision_el.find("text").text
                if len(input_text) != 0:
                    article_text = clean_article_text(revision_el.find("text").text)
                else:
                    article_text = input_text
            except:
                article_text = None
            # Set value in our dictionary

            try:
                out_dict[article_title] = dict(
                    text=article_text, id=article_id_dict[article_title]
                )
            except:
                print(f"Article title: {article_title}")

        # Now let's write this outdict to a file
        with open(mk_filename(stream_id), "w") as json_file:
            json_file.write(json.dumps(out_dict, indent=4))



def parse_xml_dump(filename: str, out_filename: str):
    """Process the entire xml dump contained in `filename`."""

    tree = ET.parse(filename)
    root = tree.getroot()

    out_dict: dict[str, str] = {}

    page_count = 0

    for page in root:

        page_count += 1
        # Now we want to get the article's name and it's raw text
        # title_el = page.find("title")
        # revision_el = page.find("revision")

        # article_title = str(title_el.text)
        # article_text = str(revision_el.find("text").text)
        # # Set value in our dictionary
        # out_dict[article_title] = article_text

    print(f"Processed {page_count} pages")
    # Now let's
