from ctypes import *

from cffi import FFI
from enum import IntEnum

from . import _native

ffi = FFI()


class CtypesEnum(IntEnum):
    """A ctypes-compatible IntEnum superclass."""

    @classmethod
    def from_param(cls, obj):
        return int(obj)


class Channel(Structure):
    pass


class Runtime(Structure):
    pass


def create_client(runtime):
    return _native.lib.create_client(runtime)


def destroy_client(client):
    _native.lib.destroy_client(client)


def create_runtime():
    return _native.lib.create_runtime()


def destroy_runtime(runtime):
    _native.lib.destroy_runtime(runtime)
