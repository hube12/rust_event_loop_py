#!/usr/bin/env python2.7
# -*- coding: utf-8 -*-

# vim: fileencoding=utf-8 filetype=python autoindent expandtab shiftwidth=4 softtabstop=4 tabstop=4

__all__ = ["PyLogLevel", "PyLogFormat", "PyLogTimeFormat", "PyLogger", "PyClient", "PyClientHandle", "PyRuntime",
           "PyMessage", "PyEvent", "PyEventType", "PyRunner", "__backend_version__"]

__version__ = "1.0.0"

# @formatter:off
from .utils             import __backend_version__                                  # noqa
from .logging           import PyLogLevel, PyLogFormat, PyLogTimeFormat, PyLogger   # noqa
from .client            import PyClient                                             # noqa
from .message           import PyMessage                                            # noqa
from .runtime           import PyRuntime                                            # noqa
from .runner            import PyRunner                                             # noqa
from .event_type        import PyEventType                                          # noqa
from .event             import PyEvent                                              # noqa
from .client_handle     import PyClientHandle                                       # noqa
# @formatter:on

if __backend_version__ != __version__:
    raise Exception("Backend version did not match the python runtime version, aborting as mismatch is ill advised")
