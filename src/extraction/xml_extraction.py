"""Extract relevant information from the entire our xml file."""

from xml.etree import ElementTree as ET


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
