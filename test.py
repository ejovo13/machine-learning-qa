from os.path import join
import json
import bz2

from icecream import ic
import wikicleaner

from src import *


def ms_process_xml():

    xml_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/xml/test.xml"
    d = ms_xml_to_dict(xml_file)

    json_out_dir = "/home/ejovo/MAIN/S9/machine_learning/project/data"

    out_filename = join(json_out_dir, "test.json")
    out_processed = join(json_out_dir, "test_processed.json")
    # Write this to a file
    with open(out_filename, "w") as json_file:
        json_file.write(json.dumps(d, indent=4))

    for article_name, article_text in d.items():
        d[article_name] = wikicleaner.clean_article_text(article_text)

    with open(out_processed, "w") as json_file:
        json_file.write(json.dumps(d, indent=4))


def single_thread_decompress():
    """Runs in about 30 seconds."""
    dump_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/dump/simplewiki-20240201-pages-articles-multistream.xml.bz2"

    out_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/xml/out.xml"

    print("Starting decompression")
    with bz2.open(dump_file) as bz2_file:
        data = bz2_file.read()

        with open(out_file, "wb") as out:
            out.write(data)

    print("All done")


def comparison():
    """Compare the before and after processing to understand what went wrong."""

    directory = "/home/ejovo/MAIN/S9/machine_learning/project/data"

    pre = join(directory, "test.json")
    post = join(directory, "test_processed.json")

    with open(pre) as pre_file:
        pre_dict = json.loads(pre_file.read())

    with open(post) as post_file:
        post_dict = json.loads(post_file.read())

    key = "Art"

    pre_value = pre_dict[key]
    post_value = post_dict[key]

    ic(pre_value)
    ic(post_value)


def process_complete_xml_dump():
    """Test function to actually process the contents of the entire xml dump"""
    xml_dump_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/xml/dump.xml"
    parse_xml_dump(xml_dump_file, None)


if __name__ == "__main__":

    # decompress_dump_multistream()
    process_multistream_xml()
    # process_complete_xml_dump()
    # ms_process_xml()
    # comparison()
