import subprocess
import time
import random
from rust_py import *


def log_it(e, t):
    # type: (PyLogLevel,PyString)->None
    print(e, t)


logger = PyLogger(log_it)
logger.configure(PyLogLevel.Info, PyLogTimeFormat.Human, PyLogFormat.Pretty)

runtime = PyRuntime()
runner = PyRunner(runtime)

client = runner.client
subscriber = runner.subscriber


def callback_event_1(event):
    print("Got event 1: {}".format(event))
    pass


def callback_event_2(event):
    print("Got event 2: {}".format(event))
    pass

child = subprocess.Popen(['exec', 'bash'], stdout=subprocess.PIPE, shell=True)
print(child.pid)
child.kill()

subscriber.subscribe(PyEventType.type_1(), callback_event_1, runtime)
subscriber.subscribe(PyEventType.type_2(), callback_event_2, runtime)

for i in range(1000):
    try:
        if random.choice([True, False]):
            msg = PyMessage.from_string("test_" + str(i))
            client.send(msg)
        else:
            msg = PyMessage.from_string(str(i))
            client.send(msg)
    except Exception as e:
        print(e)

child = subprocess.Popen(['exec', 'bash'], stdout=subprocess.PIPE, shell=True)
print(child.pid)
child.kill()
