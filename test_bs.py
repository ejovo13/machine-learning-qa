from bs4 import BeautifulSoup

def remove_html_tags(input_string):
    soup = BeautifulSoup(input_string, 'html.parser')
    for tag in soup.find_all():
        tag.decompose()
    clean_string = str(soup)
    return clean_string

if __name__ == '__main__':
    my_str = r"""\n'''Pashupatinath''' is one of the most sacred temples of Hinduism. It is in Kathmandu, the capital city of Nepal. It is the temple of the Hindu god Shiva.\n\n<center><gallery caption=\"Pashupatinath Temple\">\nPashupatinath-Ueberblick-06-2013-gje.jpg|\nPashupatinath-Tempel-13b-2013-gje.jpg|\nPashupatinath-Bagmatibruecke-08-2007-gje.jpg|\nPashupatinath-Mrigasthali-28-2013-gje.jpg|\nPashupatinath-Mrigasthali-10-Vishvarupa-2007-gje.jpg|\nPashupatinath-Verbrennung-02-Scheiterhaufen-2014-gje.jpg|\nPashupatinath-Verbrennung-16-Zeremonie-2007-gje.jpg|\nPashupatinath-Surya Ghats-96-Totengedenken-2014-gje.jpg|\n</gallery></center>\n\n\n\n\n"""

    print(my_str)

    print(remove_html_tags(my_str))