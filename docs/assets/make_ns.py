import inspect
import pkgutil
import typing as t

import nerve.tools.namespaces as namespaces

if __name__ == "__main__":
    head = """
# Namespaces

Nerve offers a rich set of predefined tools, organized in namespaces, that the agent can import [via the `using` directive](index.md#usage). This page contains the list of namespaces available in Nerve, with the descriptive prompt that will be provided to the model.
""".strip()

    print(head)
    print()

    for _, modname, _ in pkgutil.iter_modules(namespaces.__path__):
        if modname[0] != "_" and not modname.startswith("test_"):
            module = __import__(f"nerve.tools.namespaces.{modname}", fromlist=[""])

            doc = module.__doc__ or ""
            print(f"## {modname}")
            print()

            if hasattr(module, "OPTIONAL_FEATURE"):
                print("> [!IMPORTANT]")
                print(
                    f"> This namespace is not available by default and requires the `{module.OPTIONAL_FEATURE}` optional feature."
                )
                print(f"> To enable it, run `pip install nerve-adk[{module.OPTIONAL_FEATURE}]`.")
                print()

            print(doc.strip())
            print()
            print("<details>")
            print("<summary><b>Show Tools</b></summary>")
            print()

            for name, func in inspect.getmembers(module, inspect.isfunction):
                if name[0] != "_" and func.__module__ == module.__name__:
                    doc = func.__doc__ or ""
                    doc = doc.strip().strip()
                    print(f"### `{name}`")
                    print()
                    print(f"<pre>{doc}</pre>")
                    print()
                    signature = inspect.signature(func)
                    type_hints = t.get_type_hints(func, include_extras=True)
                    params = signature.parameters.items()

                    if len(params) == 0:
                        continue

                    print("**Parameters**")
                    print()

                    for param_name, _param in params:
                        if param_name == "self":
                            continue

                        param_type = type_hints.get(param_name)
                        if param_type is None:
                            print(f"**{param_name}** NO TYPE")
                            continue

                        is_annotated = t.get_origin(param_type) is t.Annotated

                        if is_annotated:
                            args = t.get_args(param_type)
                            base_type = args[0]
                            description = args[1] if len(args) > 1 else ""
                            if hasattr(description, "description"):
                                description = description.description
                        else:
                            base_type = param_type
                            description = ""

                        print(f"* `{param_name}` <i>({base_type})</i>: {description.strip()}")

            print("</details>")
            print()
