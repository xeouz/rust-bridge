from athen import register_getter

@register_getter(execution_mode = "run")
def run(query: str) -> str:
    return "Hello World!"