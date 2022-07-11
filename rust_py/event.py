#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4
from .utils import lib, Structure, PyString, ffi


class PyEvent(Structure):
    def __init__(self, event):
        # type: (PyEvent, POINTER(FFISEvent)) -> PyEvent
        super(PyEvent, self).__init__()
        # if type(event) != _native__ffi._cffi_backend._CDataBase:
        if str(type(event)) not in ["<type '_cffi_backend._CDataBase'>", "<type '_cffi_backend.CData'>"]:
            raise Exception("Not a valid pointer, got {}".format(type(event)))
        self.__inner = event

    @property
    def inner(self):
        # type: (PyEvent) -> POINTER(FFISEvent)
        return self.__inner

    def is_event_1(self):
        return self.inner.tag == lib.Event1

    def is_event_2(self):
        return self.inner.tag == lib.Event2

    def is_kill(self):
        return self.inner.tag == lib.Kill

    def __repr__(self):
        if self.is_event_1():
            return ffi.string(self.inner.event1.ptr, self.inner.event1.len)
        elif self.is_event_2():
            return ffi.string(self.inner.event2.ptr, self.inner.event2.len)
        elif self.is_kill():
            return "Kill"
        else:
            return "Unknown"
