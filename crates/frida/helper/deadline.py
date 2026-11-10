import threading
from contextlib import contextmanager
import frida

@contextmanager
def deadline(seconds):
    cancellation=frida.Cancellable()
    timer=threading.Timer(seconds,cancellation.cancel)
    timer.daemon=True
    timer.start()
    try:
        with cancellation:
            yield cancellation
    finally:
        timer.cancel()
