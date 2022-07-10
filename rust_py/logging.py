#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

from .utils import CtypesEnum, handle_error, PyString
from ctypes import Structure
from utils import lib, ffi


# @formatter:off
class PyLogLevel(CtypesEnum):
    Trace   = lib.FFILogLevelTrace          # noqa
    Debug   = lib.FFILogLevelDebug          # noqa
    Info    = lib.FFILogLevelInfo           # noqa
    Warn    = lib.FFILogLevelWarn           # noqa
    Error   = lib.FFILogLevelError          # noqa
# @formatter:on


# @formatter:off
class PyLogFormat(CtypesEnum):
    Text    = lib.FFILogFormatText          # noqa
    Json    = lib.FFILogFormatJson          # noqa
    Compact = lib.FFILogFormatCompact       # noqa
    Pretty  = lib.FFILogFormatPretty        # noqa
# @formatter:on


# @formatter:off
class PyLogTimeFormat(CtypesEnum):
    Raw     = lib.FFILogTimeFormatNone      # noqa
    Rfc3339 = lib.FFILogTimeFormatRfc3339   # noqa
    System  = lib.FFILogTimeFormatSystem    # noqa
    Human   = lib.FFILogTimeFormatUpTime    # noqa
# @formatter:on


class PyLogger(Structure):
    def __init__(self, callback, test_it=True):
        # type: (PyLogger,Callable[[PyLogLevel,PyString],None]) -> PyLogger
        """
        Create a logger from a valid callback which uses log level and a string to log it.
        String will have the error formatted as per the @{configure} method.
        Will test the callback in this constructor if test_it is True, disable it if you do not
        want it.

        :param callback: Should be a function that takes an int and a PyString as argument
        :param test_it: Boolean to indicate if the callback should be tested when the logger is built

        :return a PyLogger implementation

        :raise: Exception if test_it is True and the callback is not correct
        """
        super(PyLogger, self).__init__()
        if not callable(callback):
            raise Exception("Not a valid callback, should be Callable[[PyLogLevel,PyString],None]")
        self.__callback = callback

        def wrapper(callback_fn):
            @ffi.callback("void(FFILogLevel,FFIArray_c_uchar*)")
            def log(level, message):
                callback_fn(level, PyString(message))

            return log

        self.__wrapper = wrapper(self.__callback)

        if test_it:
            # type: POINTER(FFILogger)
            self.__inner = handle_error(lib.create_logger(self.__wrapper))
        else:
            # type: POINTER(FFILogger)
            self.__inner = lib.create_unsafe_logger(self.__wrapper)

    @property
    def inner(self):
        # type: (PyLogger) -> POINTER(FFILogger)
        return self.__inner

    def __del__(self):
        if self.__inner is not None:
            lib.destroy_pointer(self.inner)

    def configure(self, level, time_format, log_format, show_level=False, show_trace=False):
        # type: (PyLogger,PyLogLevel,PyLogTimeFormat,PyLogFormat,Optional[bool],Optional[bool]) -> PyLogger
        """
        Configure the logger output

        :param level: The log level
        :param time_format: The time format to use
        :param log_format: The produced log format
        :param show_level: If log level should be shown
        :param show_trace: If trace of Rust function should be shown

        :return: PyLogger pass as argument for chaining

        :raise: Exception if the configuration failed
        """
        config = lib.create_logger_config(level, time_format, log_format, show_level, show_trace)
        assert handle_error(lib.configure_logging(config, self.inner)) is None
        # At this point ownership of the pointer was transferred
        self.__inner = None
