#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from .utils import Structure, handle_error, lib


class PyRuntime(Structure):
    def __init__(self):
        # type: (PyRuntime) -> PyRuntime
        """
        Create a Tokio reactor that can be use later on to send any tasks onto it

        :return: A pointer to PyRuntime

        :raise: Exception with a string if the reactor can not be instantiated
        """
        super(PyRuntime, self).__init__()
        # type: POINTER(FFIRuntime)
        self.__inner = handle_error(lib.create_runtime())

    @property
    def inner(self):
        # type: (PyRuntime) -> POINTER(FFIRuntime)
        return self.__inner

    def __del__(self):
        lib.destroy_pointer(self.inner)
