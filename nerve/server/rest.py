import typing as t

import uvicorn
from fastapi import Body, FastAPI, HTTPException, Request
from loguru import logger

from nerve.models import Configuration
from nerve.runtime import Runtime
from nerve.runtime.runner import Arguments, Runner


def _get_input_state_from_request(inputs: dict[str, str], data: dict[str, str]) -> dict[str, str]:
    input_state = inputs.copy()
    for input_name in input_state.keys():
        # get user provided or default value if set
        input_value = data.get(input_name, inputs.get(input_name, None))
        if input_value is None:
            raise HTTPException(status_code=400, detail=f"input '{input_name}' is required")

        input_state[input_name] = input_value

    return input_state


def _create_agent_call_endpoint(
    run_args: Arguments,
    inputs: dict[str, str],
) -> t.Callable[[dict[str, str], Request], t.Coroutine[t.Any, t.Any, t.Any]]:
    logger.debug(f"creating request endpoint for inputs: {inputs}")

    async def _on_request(data: dict[str, str], request: Request) -> t.Any:
        # check if the "raw" query parameter is present
        raw = request.query_params.get("full", "false").lower() == "true"
        client = request.client
        client_host = ""
        if client:
            client_host = client.host

        logger.info(f"request from {client_host}: {data} [raw={raw}]")
        # validate and prepare input state from request
        input_state = _get_input_state_from_request(inputs, data)
        # create a runner
        runner = Runner(run_args, input_state)
        # execute the runner
        output = await runner.run()

        logger.debug(f"output state: {output}")

        if raw:
            return output

        return output.output

    return _on_request


def _create_tool_call_endpoint(
    tool: t.Callable[..., t.Any],
) -> t.Callable[[dict[str, str], Request], t.Coroutine[t.Any, t.Any, dict[str, t.Any]]]:
    async def _on_request(data: dict[str, str] = Body(default=None), request: Request = Request) -> dict[str, t.Any]:  # type: ignore
        client = request.client
        client_host = ""
        if client:
            client_host = client.host

        logger.info(f"request for tool {tool.__name__} from {client_host}: {data}")

        return {"result": await tool(**(data if data else {}))}

    return _on_request


def create_rest_api(
    run_args: Arguments,
    inputs: dict[str, t.Any],
    config: Configuration,
    runtime: Runtime | None,
    serve_tools: bool,
    tools_only: bool,
) -> FastAPI:
    # TODO: use Starlette instead to minimize dependencies
    app = FastAPI()

    if not tools_only:
        logger.info("🌐 creating agent endpoint")
        logger.info("  /")
        app.add_api_route(
            path="/",
            endpoint=_create_agent_call_endpoint(run_args, inputs),
            methods=["POST"],
            response_model=dict,
            summary=config.description,
        )

    if serve_tools and runtime:
        logger.info(f"🌐 creating endpoints for {len(runtime.tools)} tools")
        logger.debug(runtime.tools)

        for tool in runtime.tools:
            logger.info(f"  /{tool.__name__}")
            app.add_api_route(
                path=f"/{tool.__name__}",
                endpoint=_create_tool_call_endpoint(tool),
                methods=["POST"],
                response_model=dict,
                summary=tool.__doc__,
            )

    return app


async def serve_http_app(
    app: t.Any,
    agent_name: str,
    scheme: str,
    host: str,
    port: int,
    debug: bool,
) -> None:
    logger.info(f"🌐 serving {agent_name} on {scheme}://{host}:{port}/ ...")

    config = uvicorn.Config(app, host=host, port=port, log_level="debug" if debug else "warning")
    server = uvicorn.Server(config)

    await server.serve()
