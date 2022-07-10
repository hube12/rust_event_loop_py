#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from .utils import Structure, handle_error, lib
from .message import PyMessage
from .runtime import PyRuntime


class PyClient(Structure):
    def __init__(self, runtime):
        # type: (PyClient,PyRuntime) -> PyClient
        """
        :raise: Exception with a string if the client can not be instantiated
        """
        super(PyClient, self).__init__()
        # type: POINTER(FFIError_FFIClient)
        client = lib.create_new_client(runtime.inner)
        # type: POINTER(FFIClient)
        self.__inner = handle_error(client)
        # we store the runtime to avoid a strong reference delete before client has been shut down
        # type: PyRuntime
        self.__runtime = runtime

    @property
    def inner(self):
        # type: (FFIClient) -> POINTER(FFIClient)
        return self.__inner

    def __del__(self):
        lib.destroy_pointer(self.inner)

    def receive(self):
        # type: (PyClient) -> PyMessage
        message = lib.client_receive(self.inner)
        return PyMessage(handle_error(message))

    def send(self, message):
        # type: (PyClient, PyMessage) -> int
        recv_count = handle_error(lib.client_send(self.inner, message.consume()))
        res = int(recv_count[0])
        lib.destroy_pointer(recv_count)
        return res
