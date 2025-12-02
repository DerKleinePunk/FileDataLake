def example(*args, **kwargs):
    if args != ():
        print('called with args', args)
    if kwargs != {}:
        print('called with kwargs', kwargs)
    if args == () and kwargs == {}:
        print('called with no arguments')
