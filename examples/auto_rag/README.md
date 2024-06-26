This agent is an example of autonomous [RAG](https://blogs.nvidia.com/blog/what-is-retrieval-augmented-generation/). 

A set of text files are imported into the RAG and the model is asked a question about a piece of knowledge that is only present in one of the documents. The model will use the rag action space to retrieve it autonomously.

Using RAG requires an -E/--embedder argument to define the model used for the embeddings:

```sh
nerve \
  -G "fireworks://llama-v3-70b-instruct" \ # the model used for the agent
  -E "ollama://all-minilm@bahamut.local:11434" \ # the model used for the rag embeddings
  -T auto_rag \
  -P "define a Darmepinter"
```

Example output:

```
nerve v0.0.2 🧠 llama-v3-70b-instruct@fireworks > auto_rag
task: define a Darmepinter

[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/lorem-ipsum.txt' (813 bytes) ... done in 1.975200708s
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/made-up-animals.txt' (82 bytes) ... done in 99.349542ms
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/tiger.txt' (405 bytes) ... done in 53.671083ms

[statistics] steps:1 

[rag] What is a Darmepinter? (top 1)

  1 results in 101.144417ms
       * /Users/evilsocket/lab/nerve/examples/auto_rag/data/made-up-animals.txt (0.16502168746739188)


[statistics] steps:2 responses:1 actions:1

<memories> darmepinter-definition=A Darmepinter is a made-up animal that looks like a zebra and sounds like a snake.

task complete: 'The task of defining a Darmepinter is complete.'
```