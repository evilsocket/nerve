import inspect
import pkgutil

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
            print(doc.strip())
            print()

            print("| Tool | Description |")
            print("|------|-------------|")

            for name, func in inspect.getmembers(module, inspect.isfunction):
                if name[0] != "_" and func.__module__ == module.__name__:
                    doc = func.__doc__ or ""
                    doc = doc.strip().replace("\n", "<br>")
                    print(f"| `{name}` | <pre>{doc}</pre> |")

            print("")
