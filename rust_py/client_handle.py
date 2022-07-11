#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from .utils import Structure, handle_error, lib
from .message import PyMessage


class PyClientHandle(Structure):
    def __init__(self, handle):
        # type: (PyClientHandle,POINTER(FFIClientHandle)) -> PyClientHandle
        super(PyClientHandle, self).__init__()
        # type: POINTER(FFIClientHandle)
        self.__inner = handle

    @property
    def inner(self):
        # type: (POINTER(FFIClientHandle)) -> FFIClientHandle
        return self.__inner

    def __del__(self):
        lib.destroy_pointer(self.inner)

    def send(self, message):
        # type: (POINTER(FFIClientHandle), PyMessage) -> int
        recv_count = handle_error(lib.client_handle_send(self.inner, message.consume()))
        res = int(recv_count[0])
        lib.destroy_pointer(recv_count)
        return res
