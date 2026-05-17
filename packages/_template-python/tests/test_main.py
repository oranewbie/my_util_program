from template_python.main import greet


def test_greet():
    assert greet("world") == "Hello, world!"
