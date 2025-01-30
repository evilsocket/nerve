# image is automatically built and pushed to the docker hub at every commit to mai
FROM dreadnode/nerve:latest

WORKDIR /app
COPY task.yml .

# install required packages depending on the strike
RUN apt-get update && apt-get install -y postgresql-client wget curl nmap sqlmap

# the entrypoint is the nerve binary set by the base image

# the generator and specific prompt are supposed to be set via environment variables
CMD [ "-Ttask.yml", "--force-format"]