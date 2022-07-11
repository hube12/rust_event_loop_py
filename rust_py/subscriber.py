#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4
from .event import PyEvent
from .utils import _native, Structure, handle_error


class PySubscriber(Structure):
    def __init__(self, pointer):
        # type: (PySubscriber,POINTER(FFISubscriber)) -> PySubscriber
        super(PySubscriber, self).__init__()
        self.__inner = pointer
        self.__callbacks = []
        self.__wrappers = []

    @property
    def inner(self):
        # type: (PySubscriber) -> POINTER(FFISubscriber)
        return self.__inner

    def __del__(self):
        _native.lib.destroy_pointer(self.inner)

    def subscribe(self, event_type, callback, runtime):
        # type: (PySubscriber, PyEventType, Callable[[PyEvent],None], PyRuntime) -> None

        if not callable(callback):
            raise Exception("Not a valid callback, should be Callable[[PyEvent],None]")

        def wrapper(callback_fn):
            @_native.ffi.callback("void(FFISEvent *)")
            def receive_event(event):
                callback_fn(PyEvent(event))

            return receive_event

        wrapper = wrapper(callback)
        self.__callbacks.append(callback)
        self.__wrappers.append(wrapper)
        # this returns a FFINull, no need to check it
        assert handle_error(_native.lib.subscribe(self.inner, event_type.inner, wrapper, runtime.inner)) is None
