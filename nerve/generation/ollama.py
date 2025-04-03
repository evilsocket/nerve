import typing as t

from loguru import logger

from nerve.models import Usage


# until this is not fixed, ollama needs special treatment: https://github.com/BerriAI/litellm/issues/6353
class OllamaGlue:
    def __init__(self, api_base: str, generator_id: str, generator_params: dict[str, t.Any]) -> None:
        import ollama

        self.model = "/".join(generator_id.split("/")[1:])
        self.client = ollama.AsyncClient(host=api_base)
        self.generator_params = generator_params

        logger.debug(f"using ollama client for model {self.model}")

    async def _process_conversation(self, conversation: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        fixed = []

        """
        Ollama needs this:

        {
            'role': 'user',
            'content': [
                {'type': 'text', 'text': 'read_webcam_image returned the following response:'},
                {'type': 'image_url', 'image_url': {'url': 'data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAIBAQEBA....'}}
            ]
        }

        To be fixed into this:

        {
            'role': 'user',
            'content': 'read_webcam_image returned the following response:',
            'images': [ raw image data ]

        }
        """

        for message in conversation:
            if "content" in message and isinstance(message["content"], list):
                if (
                    len(message["content"]) == 2
                    and message["content"][0]["type"] == "text"
                    and message["content"][1]["type"] == "image_url"
                ):
                    fixed.append(
                        {
                            "role": "user",
                            "content": message["content"][0]["text"],
                            "images": [message["content"][1]["image_url"]["url"].split(",")[1]],
                        }
                    )
            else:
                fixed.append(message)

        return fixed

    async def generate(
        self, conversation: list[dict[str, t.Any]], tools_schema: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        conversation = await self._process_conversation(conversation)

        logger.debug(f"ollama.conversation: {conversation}")
        response = await self.client.chat(
            model=self.model,
            messages=conversation,
            tools=tools_schema,
            **self.generator_params,
        )
        logger.debug(f"ollama.response: {response}")
        return Usage(
            prompt_tokens=0,
            completion_tokens=0,
            total_tokens=0,
        ), response.message
