An example of an agent runinng via a Docker container that includes tools for vulnerability scanning.

Build the docker image:

```
docker build -t agent-name .
```

Run the docker container:

```
docker run -it \
    --env NERVE_GENERATOR=openai://gpt-4o \
    --env OPENAI_API_KEY=sk-your-api-key \
    --net=host \
    agent-name
```
