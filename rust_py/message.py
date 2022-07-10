#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from .utils import lib, Structure, ffi, handle_error, PyString


class PyMessage(Structure):
    def __init__(self, message):
        # type: (PyMessage, POINTER(FFIMessage)) -> PyMessage
        super(PyMessage, self).__init__()
        # type: POINTER(FFIMessage)
        self.__inner = message
        self.__string = PyString(handle_error(lib.message_as_cstring(self.__inner)))

    @classmethod
    def from_string(cls, s):
        # type: (PyMessage,str) -> PyMessage
        s = ffi.new('char[]', s.encode("utf-8"))
        return cls(handle_error(lib.message_from_cstring(s)))

    @property
    def string(self):
        return self.__string

    @property
    def inner(self):
        # type: (PyMessage) -> POINTER(FFIMessage)
        if self.__inner is None:
            raise Exception("Object was already used, you can not reuse it")
        return self.__inner

    def consume(self):
        # type: (PyMessage) -> POINTER(FFIMessage)
        inner = self.inner
        self.__inner = None
        return inner

    def __del__(self):
        if self.__inner is not None:
            lib.destroy_pointer(self.inner)

    def __str__(self):
        return self.string.__str__()

    def __repr__(self):
        return '"' + self.string.__repr__() + '"'
