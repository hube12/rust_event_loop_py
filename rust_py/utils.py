#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from ctypes import Structure, c_void_p, c_char_p
from enum import IntEnum
from . import _native, _native__ffi  # noqa

# @formatter:off
# Constants and native access
USHORT_MAX          = (1 << 16) - 1                         # noqa
UINT_MAX            = (1 << 32) - 1                         # noqa
UINT64_MAX          = (1 << 64) - 1                         # noqa
native              = _native                               # noqa
ffi                 = native.ffi                            # noqa
lib                 = native.lib                            # noqa
backend             = _native__ffi._cffi_backend            # noqa
__backend_version__ = _native.ffi.string(lib.FFI_VERSION)   # noqa
# @formatter:on


class CtypesEnum(IntEnum):
    """A ctypes-compatible IntEnum superclass."""

    @classmethod
    def from_param(cls, obj):
        return int(obj)


class FFIError(Structure):
    _pack_ = 8
    _fields_ = [("obj", c_void_p), ("error", c_char_p)]


def handle_error(ffi_error):
    if ffi_error.error == _native.ffi.NULL:
        if ffi_error.obj == _native.ffi.NULL:
            return None
        else:
            return ffi_error.obj
    exception = _native.ffi.string(ffi_error.error)
    # destroy the rust string pointer since it was given to the python backend
    _native.lib.destroy_pointer(ffi_error.error)
    raise Exception(exception)


class PyArray(Structure):
    """
    # Safety Warning
    This specific structure does take a pointer as a parameter as for duck typing we
    need to return an empty type. However if you give a wrong pointer which does not
    allocate a vector behind the scene, this will crash hard.

    This owning structure does deallocate the vector that was to create the inner
    structure in the FFI part and also the pointer.
    """

    def __init__(self, array):
        # type: (PyArray,POINTER(FFIArray)) -> PyArray
        super(PyArray, self).__init__()
        self.__inner = _native.ffi.unpack(array[0].ptr, array[0].len)
        # type: POINTER(FFIArray)
        self.__ffi_array = array

    @property
    def inner(self):
        # type: (PyArray) -> [POINTER(Any)]
        """
        :return: A list of pointer to the underlying object (can not be known due to duck typing)
        """
        return self.__inner

    def __del__(self):
        # This destroy both the vector and the pointer
        _native.lib.destroy_array(self.__ffi_array)

    def __str__(self):
        return self.inner

    def __repr__(self):
        return self.inner


class PyString(PyArray):
    def __init__(self, array):
        # type: (PyString,POINTER(FFIArray_c_uchar)) -> PyString
        super(PyString, self).__init__(array)

    def to_string(self):
        # type: (PyString) -> str
        return bytearray(self.inner).decode("utf-8")

    def to_bytes(self):
        # type: (PyString) -> List[int]
        return self.inner

    def __str__(self):
        return self.to_string()

    def __repr__(self):
        return self.to_string()


def is_u16(i):
    return i is not None and isinstance(i, int) and 0 <= i < USHORT_MAX


def is_u32(i):
    return i is not None and isinstance(i, int) and 0 <= i < UINT_MAX


def is_u64(i):
    return i is not None and isinstance(i, int) and 0 <= i < UINT64_MAX
