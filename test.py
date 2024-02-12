from os.path import join
import json
import bz2

from icecream import ic
import wikicleaner

from src import *


def multi_streaming():

    dump_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/dump/simplewiki-20240201-pages-articles-multistream.xml.bz2"
    open_dump(dump_file)

    index_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/dump/simplewiki-20240201-pages-articles-multistream-index.txt"

    indices = open_index(index_file)

    # Now we should try and read from one index to the next
    # start_index = indices[0]
    # next_index = indices[1]
    # ic((start_index, next_index))
    # xml_bytes = read_dump_stream(start_index, next_index, dump_file)

    # print(xml_bytes[0:100])
    # print("...")
    # print(xml_bytes[-100:])
    # print(len(indices))

    # destination_folder = "/home/ejovo/MAIN/S9/machine_learning/project/data/xml"

    # # Now let's generate some xml files
    # xml_filename = join(destination_folder, "test.xml")
    # with open(xml_filename, 'wb') as xml_out:
    #     xml_out.write(b"<root>\n")
    #     xml_out.write(xml_bytes)
    #     xml_out.write(b"</root>")

    # Ok, now we have the skeleton that will process this data in a single unit.
    # We can go ahead and launch multiple subprocesses to generate the sql in parallel.

    # Let's just try sequentially to see how our program runs.
    # for (idx, byte_idx) in enumerate(indices[:-1]):


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

    process_complete_xml_dump()
    # ms_process_xml()
    # comparison()
