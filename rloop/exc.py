import traceback
from asyncio.log import logger as _aio_logger


def _default_exception_handler(context):
    message = context.get('message')
    if not message:
        message = 'Unhandled exception in event loop'

    exception = context.get('exception')
    if exception is not None:
        exc_info = (type(exception), exception, exception.__traceback__)
    else:
        exc_info = False

    # if ('source_traceback' not in context and
    #         self._current_handle is not None and
    #         self._current_handle._source_traceback):
    #     context['handle_traceback'] = \
    #         self._current_handle._source_traceback

    log_lines = [message]
    for key in sorted(context):
        if key in {'message', 'exception'}:
            continue
        value = context[key]
        if key == 'source_traceback':
            tb = ''.join(traceback.format_list(value))
            value = 'Object created at (most recent call last):\n'
            value += tb.rstrip()
        elif key == 'handle_traceback':
            tb = ''.join(traceback.format_list(value))
            value = 'Handle created at (most recent call last):\n'
            value += tb.rstrip()
        else:
            value = repr(value)
        log_lines.append(f'{key}: {value}')

    _aio_logger.error('\n'.join(log_lines), exc_info=exc_info)


def _exception_handler(context, handler):
    if handler is None:
        try:
            _default_exception_handler(context)
        except (KeyboardInterrupt, SystemExit):
            raise
        except BaseException:
            _aio_logger.error('Exception in default exception handler', exc_info=True)
    else:
        try:
            handler(context)
        except (KeyboardInterrupt, SystemExit):
            raise
        except BaseException as exc:
            # Exception in the user set custom exception handler.
            try:
                # Let's try default handler.
                _default_exception_handler(
                    {
                        'message': 'Unhandled error in exception handler',
                        'exception': exc,
                        'context': context,
                    }
                )
            except (KeyboardInterrupt, SystemExit):
                raise
            except BaseException:
                # Guard 'default_exception_handler' in case it is
                # overloaded.
                _aio_logger.error(
                    'Exception in default exception handler '
                    'while handling an unexpected error '
                    'in custom exception handler',
                    exc_info=True,
                )
