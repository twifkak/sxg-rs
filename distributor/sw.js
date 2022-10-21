self.addEventListener('fetch', event => {
  let {request} = event;
  let url = new URL(request.url);
  let {pathname} = url;
  if (pathname.startsWith('/doc/') || pathname.startsWith('/crt/') || pathname.startsWith('/sub/')) {
    event.respondWith(fetch(request));
  }
});
