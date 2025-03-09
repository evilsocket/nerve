import inspect
import pkgutil


def print_namespaces() -> None:
    import nerve.tools.namespaces as namespaces

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
