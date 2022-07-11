#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4
from .utils import _native, Structure, handle_error
from .runtime import PyRuntime
from .subscriber import PySubscriber
from .client_handle import PyClientHandle



class PyRunner(Structure):
    def __init__(self, runtime):
        # type: (PyRunner,PyRuntime) -> PyRunner
        super(PyRunner, self).__init__()
        # type: POINTER(FFIError_FFIRunner)
        runner = _native.lib.create_new_runner(runtime.inner)
        # type: POINTER(FFITuple_FFIRunner_FFISubscriber)
        tuple = handle_error(runner)
        # type: POINTER(FFIAuditRunner)
        self.__inner = handle_error(_native.lib.runner_get_runner(tuple))
        # type: PySubscriber
        self.__subscriber = PySubscriber(handle_error(_native.lib.runner_get_subscriber(tuple)))
        # type: PyClient
        self.__client = PyClientHandle(handle_error(_native.lib.runner_get_client(tuple)))
        # we store the runtime to avoid a strong reference delete before client has been shut down
        # type: PyRuntime
        self.__runtime = runtime

    @property
    def inner(self):
        # type: (PyRunner) -> POINTER(FFIRunner)
        return self.__inner

    @property
    def subscriber(self):
        # type: (PyRunner) -> PySubscriber
        return self.__subscriber

    @property
    def client(self):
        # type: (PyRunner) -> PyClient
        return self.__client

    def __del__(self):
        _native.lib.destroy_pointer(self.inner)
