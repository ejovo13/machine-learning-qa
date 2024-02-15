"""Extract relevant information from the entire our xml file."""

import os
import json
from bs4 import BeautifulSoup
import os.path as os_path
from xml.etree import ElementTree as ET
from .bz2_extraction import open_index
from .const import *


import wikicleaner as wc


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

    article_hashes = set()

    for stream_id, xml_file in enumerate(os.listdir(xml_directory), 1):

        # if stream_id != 4707: continue

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

            # Skip some extremely f'ed up articles
            if article_title in [
                "Wikipedia:WikiProject Check Wikipedia/Translation",
                "Template:Template usage",
                "Module:Escape/doc",
            ]:
                continue

            if ":" in article_title:
                continue

            input_text: str = revision_el.find("text").text
            if len(input_text) == 0:
                continue

            # Skip redirects
            if r"#REDIRECT" in input_text[:10]:
                continue

            article_text = process_article_text(input_text)
            article_hash = hash(article_text)

            # Avoid redundant articles
            if article_hash not in article_hashes:
                article_hashes.add(article_hash)
            else:
                continue

            out_dict[article_title] = dict(
                text=article_text,
                id=article_id_dict[article_title],
                text_hash=article_hash,
            )

        # Now let's write this outdict to a file
        with open(mk_filename(stream_id), "w") as json_file:
            json_file.write(json.dumps(out_dict, indent=4))


def remove_html_tags(input_string):
    """Remove all html tags and their inner content from our text.

    Thanks chatgpt..
    """
    soup = BeautifulSoup(input_string, "html.parser")
    for tag in soup.find_all():
        tag.decompose()
    clean_string = str(soup)
    return clean_string


def process_article_text(article_text: str) -> str:
    """Process the actual article text string in our `"text"` node."""
    if len(article_text) != 0:
        article_text = wc.clean_article_text(article_text)
        article_text = remove_html_tags(article_text)
        article_text = wc.post_processing(article_text)

    return article_text


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
