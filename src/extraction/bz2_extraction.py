"""Routines to extract certain information from the bz2 archive."""

import bz2

DEFAULT_DUMP_FILE = "/home/ejovo/MAIN/S9/machine_learning/project/data/dump/simplewiki-20240201-pages-articles-multistream.xml.bz2"


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


def open_index(filename: str) -> list[int]:
    """Return a list of indexes that designate the different multistreams in our data dump."""
    indices = set()
    line_count = 0

    with open(filename, "r") as file:

        for line in file:
            line_count += 1
            index = line.split(":")[0]
            indices.add(int(index))

    print(f"Read: {line_count} lines")
    return sorted(list(indices))


def decompress_dump_multistream(
    filename: str = DEFAULT_DUMP_FILE, output_directory: str = DEFAULT_MULTISTREAM_OUT
):
    """Decompress the content of our dump into different xml files, one for each stream"""
