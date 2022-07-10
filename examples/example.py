from rust_py import *


def log_it(e, t):
    # type: (PyLogLevel,PyString)->None
    print(e, t)


logger = PyLogger(log_it)
logger.configure(PyLogLevel.Info, PyLogTimeFormat.Human, PyLogFormat.Pretty)

runtime = PyRuntime()
client = PyClient(runtime)

for i in range(1000):
    msg = PyMessage.from_string("test")
    client.send(msg)
    client.receive()
