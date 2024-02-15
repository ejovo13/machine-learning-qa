# Project

### Preliminaries

- Start off by downloading the entire wikipedia database (simple english)
- 300MB compressed
- Comes in a format with multistream bz2 compression

The first thing we had to do was parse this information into a workable format.
Once that was done, we were able to start encoding this information in a local embedding using chromadb's python API.


#### Multistream

The data dump consists of multiple XML files that are compressed using the Bzip2 compression algorithm. These files are concatenated into a single datadump and are accompanied by a text file indicating where indidividual articles begin and end.

> These files are accompanied by an index, which has the same name as the multistream file but with the added "-index.txt" in the name. The index consists of lines of the format file-offset:page-id:page-title where the file offset is the position in the multistream file of the start of the stream containing the specific page id and page title.


#### Wikipedias XML format

Another important step was to understand the XML schema that wikipedia uses to store their information


## Challenges faced in this first part

- The original `Bz2.read` function was automatically decompressing and returning n *uncompressed* bytes, whereas the wikipedia indices were referring to compressed bytes.
- The xml strings returned had many `pages`, thus they were not able to be parsed. Somehow I need to split up these files.

- Realistically these challenges only need to be addressed for a much larger dataset.
- Using the `bz2.decompress` method extracted a multistream file in a single pass (30s).
    - Decompressed, we end up with 1.21GB

#### Processing the XML

Now we want to go ahead and extract only the important information from this large xml file

I think that's why we wanted to use an isolated test, so that we can examine the format and extract only relevant information.

###### Keeping only the most recent revision

Each individual wikipedia page has many revisions. We only want the latest revision (or a revision with a specific cutoff date!) to be used in our embedding.

###### Removing images

Files can be linked in wikipedia with a special markdown syntax. Example:
```
[[File:Colorful spring garden.jpg|thumb|180px|right|[[Spring]] flowers in April in the [[Northern Hemisphere]].]]
```

We want to remove any linked files for our embedding.

###### Removing links

Links to other wikipedia pages are created using `[[Another Article]]`. Since the words contained are still important to the meaning of the text, we only want to remove the double brackets (`[[Another Article]] => Another Article`)

###### Other elements
- ``{{monththisyear|4}}``


Processing the XML was actually the most time consuming portion of the project

We later discovered that there is a data set onlin


##### Beautiful soup

```
/usr/lib/python3.11/html/parser.py:170: XMLParsedAsHTMLWarning: It looks like you're parsing an XML document using an HTML parser. If this really is an HTML document (maybe it's XHTML?), you can ignore or filter this warning. If it's XML, you should know that using an XML parser will be more reliable. To parse this document as XML, make sure you have the lxml package installed, and pass the keyword argument `features="xml"` into the BeautifulSoup constructor.
  k = self.parse_starttag(i)
```


### Encoding Pages Into Embeddings

We used the ChromaDB library with Python bindings to efficiently store the content of wikipedia articles in text.