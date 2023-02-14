import http.server
import os
import sys
import threading
import webbrowser

class CustomDirectoryHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        kwargs['directory'] = os.getcwd() + os.sep + 'web'
        super().__init__(*args, **kwargs)

addr = ('', 8000)
handler = CustomDirectoryHandler
with http.server.ThreadingHTTPServer(addr, handler) as httpd:
    try:
        t = threading.Timer(1, webbrowser.open, kwargs={'url': '127.0.0.1:8000'})
        t.start()
        httpd.serve_forever()
    except KeyboardInterrupt:
        sys.exit(0)
