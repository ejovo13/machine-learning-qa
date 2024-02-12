"""Routines to extract certain information from the bz2 archive."""

import bz2
import html
from dataclasses import dataclass
import os.path as os_path

from .const import *


def open_dump(filename: str = DEFAULT_DUMP_FILE):

    file = bz2.open(filename, "rb")
    print(f"Opened new file: {file}")
    print(f"| seekable: {file.seekable()}")
    file.close()


def read_dump_stream(
    start_index: int, end_index: int, filename: str = DEFAULT_DUMP_FILE
) -> bytes:
    """Try and decompress a dump file starting from bytes `start_index` to `end_index

    Parameters
    ----------
    start_index : int
        the file position (in bytes) where to start reading from.
    end_index : int
        the file position (exclusive; in bytes) where to stop reading.
    """
    num_bytes_to_read = end_index - start_index

    file = open(filename, "rb")
    file.seek(start_index)
    stream_bytes = file.read1(num_bytes_to_read)

    return bz2.decompress(stream_bytes)


@dataclass
class IndexEntry:
    """Encodes article metadata parsed from 'pages-articles-multistream-index.txt'"""

    multistream_index: int
    article_id: int
    article_title: str


def open_index(
    filename: str = DEFAULT_INDEX_FILE,
) -> tuple[list[IndexEntry], list[int]]:
    """Return a list of IndexEntry and byte indices that designate the different multistreams in our data dump."""
    byte_indices = set()
    index_entries: list[IndexEntry] = []
    line_count = 0

    with open(filename, "r") as file:

        for line in file:
            # Keep track of the set of actual byte indices
            first_colon_idx = line.find(":")
            second_colon_idx = line.find(":", first_colon_idx + 1)

            byte_index = int(line[0:first_colon_idx])
            article_id = int(line[first_colon_idx + 1:second_colon_idx])
            article_title = line[second_colon_idx + 1:].strip()
            article_title.replace("&amp;", "&")
            article_title = html.unescape(article_title)

            byte_indices.add(byte_index)
            # Store metainformation about each index
            index_entries.append(
                IndexEntry(len(byte_indices), article_id, article_title)
            )
            line_count += 1

    print(f"Processed {line_count} lines")
    byte_indices = sorted(list(byte_indices))
    return index_entries, byte_indices


def write_bytes(bs: bytes, filename: str):
    with open(filename, "wb") as file:
        file.write(bs)


def write_xml_bytes(bs: bytes, filename: str):
    """Write bytes to an xml file, opening and closing with `root` element"""
    with open(filename, "wb") as file:
        file.write(b"<root>\n")
        file.write(bs)
        file.write(b"</root>")


def decompress_dump_multistream(
    dump_file: str = DEFAULT_DUMP_FILE,
    index_file: str = DEFAULT_INDEX_FILE,
    output_directory: str = DEFAULT_MULTISTREAM_XML_DIR,
):
    """Decompress the content of our dump - in serial - into different xml files, one for each stream.

    This function runs in about 30 seconds.
    """
    # Start off by getting metadata about our indices
    entries, byte_indices = open_index(index_file)
    print(f"Found {len(byte_indices)} indices.")

    # Now we want to extract a single byte stream, starting from one index to the next.
    total_bytes = os_path.getsize(dump_file)
    total_streams = entries[-1].multistream_index
    # Quick lambdas
    mk_filename = lambda stream_index: os_path.join(
        output_directory, "{:04}.xml".format(stream_index)
    )

    # First start with a pre and post
    print(f"Total streams: {total_streams}")

    # Now let's sequentially iterate through all of our different indices
    # pre
    write_bytes(read_dump_stream(0, byte_indices[0]), mk_filename(0))
    # meat
    for (start_idx, end_idx, stream_id) in zip(
        byte_indices[0:-1], byte_indices[1:], range(1, len(byte_indices))
    ):
        write_xml_bytes(read_dump_stream(start_idx, end_idx), mk_filename(stream_id))
        if stream_id % 500 == 0:
            print("500 streams extracted..")
    # post
    write_bytes(
        read_dump_stream(byte_indices[-1], total_bytes), mk_filename(len(byte_indices))
    )
