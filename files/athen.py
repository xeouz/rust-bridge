import athen_rs

def register_getter(func=None, execution_mode=""):
    if func:
        return athen_rs.register_func(func, "run")
    else:
        def wrapper(function):
            return athen_rs.register_func(function, execution_mode)
        return wrapper