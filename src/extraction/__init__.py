"""Scripts to extract the information from wikipedia's dumps."""

from .bz2_extraction import open_dump, read_dump_stream, open_index
from .xml_extraction import ms_xml_to_dict, parse_xml_dump
