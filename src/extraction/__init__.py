"""Scripts to extract the information from wikipedia's dumps."""

from .bz2_extraction import decompress_dump_multistream
from .xml_extraction import ms_xml_to_dict, parse_xml_dump, process_multistream_xml
