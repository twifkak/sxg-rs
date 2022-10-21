self.addEventListener('fetch', event => {
  let {request} = event;
  let url = new URL(request.url);
  let {pathname} = url;
  if (pathname.startsWith('/doc/') || pathname.startsWith('/crt/') || pathname.startsWith('/sub/')) {
    event.respondWith((async () => {
      let cache = await caches.open('sxg');
      let response = await cache.match(request);
      if (!response) {
        response = await fetch(request);
        await cache.put(request, response);
      }
      return response;
    })());
  }
});
