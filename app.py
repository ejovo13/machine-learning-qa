"""The actual Question-Answer application code."""

from pydantic_settings import BaseSettings
from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage
import polars as pl
from collections.abc import Callable
from langchain.tools import DuckDuckGoSearchResults
from chromadb import PersistentClient
from nltk.tokenize import word_tokenize

# ---------------------------------------------------------------------------- #
#                     Settings for environemental variables                    #
# ---------------------------------------------------------------------------- #
class Settings(BaseSettings):
    """Configuration settings for our application."""

    OPENAI_API_KEY: str
    MODEL_ASK: str
    MODEL_EVAL: str

    class Config:
        """Nested class to define path of .env file"""

        env_file = ".env"
        env_file_encoding = "utf-8"
        extra = "ignore"

# Langchain models
settings = Settings(_env_file=".env")
chat_ask = ChatOpenAI(model=settings.MODEL_ASK, api_key=settings.OPENAI_API_KEY, temperature=0.0)
chat_eval = ChatOpenAI(model=settings.MODEL_EVAL, api_key=settings.OPENAI_API_KEY, temperature=0.0)
search = DuckDuckGoSearchResults()
client = PersistentClient()
simple = client.get_collection("simple_wikipedia")


# ---------------------------------------------------------------------------- #
#                                 Util Function                                #
# ---------------------------------------------------------------------------- #
def append_question_mark(question: str) -> str:
    if question[-1] != "?": question += "?"
    return question

def num_tokens(input_str: str) -> int:
    """Count the number of tokens in `input_str` using `nltk`."""
    return len(word_tokenize(input_str))

# def limit_tokens(input_str: str, max_token_limit: int = 3500) -> str:
#     """Limit the number of tokens in our input string."""
#     tokenized = word_tokenize(input_str)
#     num_tokens = len(tokenized)

#     if num_tokens > max_token_limit

from duckduckgo_search import DDGS
ddgs = DDGS()


# ---------------------------------------------------------------------------- #
#                         Functions to retrieve context                        #
# ---------------------------------------------------------------------------- #
def retrieve_context_ddg(question: str, n_results: int = 3, max_tokens: int = 3500) -> str:
    """Using the DuckDuckGo search engine, retrieve context"""

    context = ""

    for r in ddgs.text(keywords=question, region="en-us", max_results=n_results):
        try:
            combined_str = f"title: {r['title']} \nbody: {r['body']}"
            next_context = context + combined_str
            if num_tokens(next_context) < max_tokens:
                context = next_context
                continue
            else:
                break

        except:
            pass

    return context

def retrieve_context_chroma(question: str, n_results: int = 3, max_tokens: int = 3500) -> str:
    context = ""

    result = simple.query(query_texts=question, n_results=n_results)

    try:
        for (metadata, document) in zip(result["metadatas"][0], result["documents"][0]):

            combined_str = f"title: {metadata['title']} \nbody: {document}"
            next_context = context + combined_str
            if num_tokens(next_context) < max_tokens:
                context = next_context
                continue
            else:
                break

    except:
        pass

    return context



# ---------------------------------------------------------------------------- #
#                            Ask question callbacks                            #
# ---------------------------------------------------------------------------- #
def ask_question_no_rag_no_instructions(question: str) -> str:
    """Ask a question to our base model using no RAG and adding no additional instructions"""
    message = HumanMessage(content=append_question_mark(question))
    return chat_ask.invoke([
        message
    ]).content


def ask_question_no_rag(question: str) -> str:
    """Ask our chat model a question and return the response."""

    question = append_question_mark(question)

    instructions = """I will ask a question, without context, and you are to provide the _most concise_ answer possible. For example, if I ask you 'Which band sings "Paint it black"?', you must respond 'The Rolling Stones'. Assume that the current year is 2019. When it comes to sports years, answer with the year of the _season_."""

    prompt = f"""{instructions} Finally, here is the question: {question}"""
    message = HumanMessage(content=prompt)

    return chat_ask.invoke([
        message
    ]).content

def ask_question_rag(question: str, context: str) -> str:
    """Ask our chat model a question with some additional context."""

    question = append_question_mark(question)

    instructions = """I will ask a question, with context, and you are to provide the _most concise_ answer possible. For example, if I ask you 'Which band sings "Paint it black"?', you must respond 'The Rolling Stones'. Assume that the current year is 2019. When it comes to sports years, answer with the year of the _season_. It is important to first consider the following context when responding: """

    num_tokens = len(word_tokenize(context))

    assert num_tokens < 3500, f"Context too long! ({num_tokens} tokens)"


    pre_prompt = instructions + context
    prompt = f"""{pre_prompt}
              Finally, here is the question: {question}"""
    message = HumanMessage(content=prompt)

    return chat_ask.invoke([
        message
    ]).content


def ask_question_chroma(question: str, n_results: int = 3, max_tokens: int = 3500) -> str:
    """Ask our chat model a question send along additional context"""
    context = retrieve_context_chroma(question, n_results, max_tokens)
    return ask_question_rag(question, context)

def ask_question_ddg(question: str, n_results: int = 3, max_tokens: int = 3500) -> str:
    """Ask our chat model a question send along additional context"""
    context = retrieve_context_ddg(question, n_results, max_tokens)
    return ask_question_rag(question, context)


QA_EVAL_CSV = "./qa_eval.csv"
import os

# ---------------------------------------------------------------------------- #
#                                Retrieving Data                               #
# ---------------------------------------------------------------------------- #
def get_qa_data() -> pl.DataFrame:
    """Return the Question and Answering _evaluation_ dataset as a dataframe."""

    if not os.path.exists(QA_EVAL_CSV):

        from datasets import load_dataset
        ds = load_dataset("nq_open", split="validation")
        questions: list[str] = ds["question"]
        answers: list[list[str]] = ds["answer"]
        answers_joined = [";".join(ans) for ans in answers]

        df = pl.DataFrame(dict(
            question=questions,
            answer=answers_joined
        ))

        df.write_csv(QA_EVAL_CSV)
        return df

    else:

        return pl.read_csv(QA_EVAL_CSV)


# ---------------------------------------------------------------------------- #
#                             Evaluation functions                             #
# ---------------------------------------------------------------------------- #
# def evaluate_question_model(questions: list[str], expected_answers: list[list[str]], question_callback: Callable[[str], str]) -> pl.DataFrame:
#     """Evaluate our question callback, returning a dataframe containing the expected answers and the actual answers."""

#     # Iterate through the questions
#     expected_answers_joined = [";".join(answers) for answers in expected_answers]
#     models_answers = []

#     for question in questions:
#         print(f"Asking '{question}'")
#         model_answer = question_callback(question)


#         models_answers.append(model_answer)

#     return pl.DataFrame(dict(
#         question=questions,
#         answer=models_answers,
#         correct_answers=expected_answers_joined
#     ))

def retrieve_answers(df_q_and_a: pl.DataFrame, question_callback: Callable[[str], str], limit: int = 10) -> pl.DataFrame:
    """Retrieve the answers from our model."""
    assert "question" in df_q_and_a, "Dataframe must have column: 'question'"

    limited_df = df_q_and_a[:limit]
    models_answer = []

    for row in limited_df.rows(named=True):
        question = row["question"]
        print(f"Asking '{question}'")
        model_answer = question_callback(question)
        models_answer.append(model_answer)

    models_answer = pl.Series(models_answer)

    return limited_df.with_columns(model_answer=models_answer)

def evaluate_question_model(df_q_and_a: pl.DataFrame, question_callback: Callable[[str], str], evaluation_callback: Callable[[str, str, str], bool], limit: int = 10) -> pl.DataFrame:
    """Evaluate questions when given a dataframe with two columns: question and answer."""

    assert "question" in df_q_and_a, "Dataframe must have column: 'question'"

    limited_df = df_q_and_a[:limit]
    models_answer = []
    are_equal_list = []

    for row in limited_df.rows(named=True):
        question = row["question"]
        correct_answer = row["answer"]

        print(f"Asking '{question}'")
        model_answer = question_callback(question)
        models_answer.append(model_answer)
        are_equal_list.append(evaluation_callback(question, model_answer, correct_answer))

    models_answer = pl.Series(models_answer)
    are_equal = pl.Series(are_equal_list)

    return limited_df.with_columns(model_answer=models_answer, are_equal=are_equal)

def convert_question_into_query(question: str) -> str:
    """Given a question, return a string that can be used in a search engine for optimal results."""

    instructions = """You are about to receive a question. You are to convert the question into a
                      concise string that can be used in a search engine to get the most relevant results.
                      For example, 'How old is Barack Obama' should be converted to 'Barack Obama age'.
                      We will use DuckDuckGo as the search engine, so don't put any nasty quotation marks
                      in your answer.

                      Here's the real question: """

    message = HumanMessage(content=instructions + question)

    return chat_ask.invoke([
        message
    ]).content


# ---------------------------------------------------------------------------- #
#                              Are Equal Functions                             #
# ---------------------------------------------------------------------------- #
def are_equal_llm(question: str, model_answer: str, correct_answer: str, verbose: bool = False) -> bool:
    """Evaluation callback using our OpenAI model to determine if two answers are equal."""

    instructions = f"""I am going to provide you with a question, and two answers. You are
                      to determine if the two answers are semantically the same (yes) or if
                      they are semantically different (no). You must ONLY respond with 'yes' or 'no'

                      For example, '1972' and '14 December 1972' are semantically different,
                      because the second answer contains more information. However, '14 December 1972'
                      and 'December 14th, 1972' are syntactically different but SEMANTICALLY the same.

                      Here is the question: '{question}'
                      Here is the first answer: '{model_answer}'
                      Here is the second answer: '{correct_answer}'

                      Do these two answers effectively mean the same thing?
                   """

    if verbose: print(f"Asking: {instructions}")

    message = HumanMessage(content=instructions)

    response_str = chat_eval.invoke([message]).content

    response_trimmed = response_str.strip('.').lower()

    if verbose: print(f"Response: {response_trimmed}")

    return response_trimmed == "yes"


import random

def are_equal_human(question: str, model_answer: str, correct_answer: str) -> bool:
    """Evaluation callback using a human's input to determine if two answers are equal"""

    if random.random() > 0.5:
        a1 = correct_answer
        a2 = model_answer
    else:
        a1 = model_answer
        a2 = correct_answer



    p1 = f"Question: '{question}'\n"
    p2 = f"Answer 1: '{a1}'\n"
    p3 = f"Answer 2: '{a2}'\n"
    p4 = "Are these answers equal? Y/n"
    prompt = p1 + p2 + p3 + p4

    inp = input(prompt).strip().lower()

    return  inp == "" or inp == "y"

def search_with_ddg(query: str, postfix: str = "AND wikipedia") -> str:
    results = search.run(query + postfix)
    return results