This agent is an example of autonomous [RAG](https://blogs.nvidia.com/blog/what-is-retrieval-augmented-generation/). 

A set of text files are imported into the RAG and the model is asked a question about a piece of knowledge that is only present in one of them. The model will use the rag action space to retrieve it autonomously.

Using RAG requires an additional `-E/--embedder` argument to define the model used for the embeddings:

```sh
nerve \
  -G "fireworks://llama-v3-70b-instruct" \ # the model used for the agent
  -E "ollama://all-minilm@localhost:11434" \ # the model used for the rag embeddings
  -T auto_rag \
  -P "define a Darmepinter"
```

Example output:

```
nerve v0.0.2 🧠 llama-v3-70b-instruct@fireworks > auto_rag
task: define a Darmepinter

[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/bear.txt' (599 bytes) ... time=562.460542ms embedding_size=384
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/horse.txt' (941 bytes) ... time=61.978125ms embedding_size=384
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/lorem-ipsum.txt' (813 bytes) ... time=141.439417ms embedding_size=384
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/made-up-animals.txt' (82 bytes) ... time=126.579125ms embedding_size=384
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/mouse.txt' (389 bytes) ... time=58.670083ms embedding_size=384
[rag] indexing document '/Users/evilsocket/lab/nerve/examples/auto_rag/data/tiger.txt' (405 bytes) ... time=98.503583ms embedding_size=384
...
...

[rag] What is a Darmepinter? (top 1)

  1 results in 15.338708ms
       * /Users/evilsocket/lab/nerve/examples/auto_rag/data/made-up-animals.txt (0.16502168746739287)

<memories> Darmepinter-definition=A Darmepinter is a made-up animal that looks like a zebra and sounds like a snake.

task complete: 'The task of defining a Darmepinter is complete.'
```