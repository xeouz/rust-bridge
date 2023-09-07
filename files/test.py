import rustmodule
import sys

data = "<Y DATA"

def register(func=None, mode=""):
    if func:
        return rustmodule.register_func(func, "run")
    else:
        def wrapper(function):
            return rustmodule.register_func(function, mode)
        return wrapper

@register(mode = "init")
def init():
    global data
    data = "EWEWEEE"

@register(mode = "run")
def run(q):
    return data