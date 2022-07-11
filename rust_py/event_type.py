#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4
from .utils import Structure, lib, ffi


class PyEventType(Structure):
    def __init__(self, event_type):
        # type: (PyEventType, SEventType) -> PyEventType
        super(PyEventType, self).__init__()
        if type(event_type) != int:
            raise Exception("Not a valid int, got {}".format(type(event_type)))
        self.__inner = event_type

    @property
    def inner(self):
        # type: (PyEventType) -> POINTER(SEventType)
        return self.__inner

    @classmethod
    def type_1(cls):
        # type: (PyEventType) -> PyEventType
        return cls(lib.EventType1)

    @classmethod
    def type_2(cls):
        # type: (PyEventType) -> PyEventType
        return cls(lib.EventType2)
