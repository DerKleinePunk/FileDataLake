def example(*args, **kwargs) -> dict:
    if args != ():
        print('called with args', args)
    if kwargs != {}:
        print('called with kwargs', kwargs)
    if args == () and kwargs == {}:
        print('called with no arguments')

    newdict = {"test1":"value1", "test2":"value2", "test3":"value3"}
    return newdict
