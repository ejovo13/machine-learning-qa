# Wikicleaner

Wikipedia provides a series of [dumps](https://dumps.wikimedia.org/backup-index.html) in XML format. While developing this project, I personally used the [simple wiki](https://dumps.wikimedia.org/simplewiki/20240201/) dumps.

## Purpose

The intent of this project is to provide some low-level routines that allow us to manipulate the content of XML dumps. One such routine removes all of the annotations like `[[File:*]]` and `{{some metadata}}` in order to make our input more digestible for an AI application that uses embeddings.

## Example Usage

```py
>>> import wikicleaner as wc
>>> article_raw_text = "[[File:Colorful spring garden.jpg|thumb|180px|right|[[Spring]] flowers in April in the [[Northern Hemisphere]].]] April comes between [[March]] and [[May]]"
>>> wc.clean_article_text(article_raw_text)
' April comes between March and May'
```