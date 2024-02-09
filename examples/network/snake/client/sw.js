self.addEventListener('fetch', function (event) {
    event.respondWith(caches.open('FETCH').then(async (cache) => {
        const cachedResponse = await cache.match(event.request);
        const fetchedResponse = fetch(event.request).then((networkResponse) => {
            cache.put(event.request, networkResponse.clone());
            return networkResponse;
        });

        return cachedResponse || fetchedResponse;
    }));
});