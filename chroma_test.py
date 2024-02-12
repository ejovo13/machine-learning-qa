"""Functions for playing around with chroma db embeddings."""

from chromadb.utils import embedding_functions
from chromadb import PersistentClient
import json
from nltk.tokenize import word_tokenize

default_ef = embedding_functions.DefaultEmbeddingFunction()
default_collection = "simple_wikipedia"


def test_article_embedding():
    """Try and embed a single article and then play around with it to learn ChromaDB's API."""

    chroma_client = PersistentClient()

    current_collections = chroma_client.list_collections()
    collection_names = [c.name for c in current_collections]

    if not default_collection in collection_names:
        chroma_client.create_collection(default_collection)

    current_collections = chroma_client.list_collections()
    collection_names = [c.name for c in current_collections]
    print("Working with the following collections:")
    print(f" {collection_names}")

    json_file = "/home/ejovo/MAIN/S9/machine_learning/project/data/test_processed.json"

    with open(json_file) as file:
        articles = json.loads(file.read())

    art_text = articles["Art"]

    # Tokenizing
    # art_tokenized = word_tokenize(art_text)
    # print(f"Number of tokens in art: {len(art_tokenized)}")
    # print(art_tokenized)
    # How many tokens in art_text?

    # Let's try and add metadata to our db
    wikipedia_collection = current_collections[0]

    wikipedia_collection.add(
        documents=art_text, metadatas={"article_title": "Art"}, ids="1"
    )


def test_retrieval():
    """Play around with chroma's API to retrieve a document."""
    chroma_client = PersistentClient()
    wikipedia_collection = chroma_client.list_collections()[0]

    result = wikipedia_collection.query(query_texts="What is the meaning of Art")

    print(result)


if __name__ == "__main__":

    # test_article_embedding()
    test_retrieval()
