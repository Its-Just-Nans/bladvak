var cacheName = "BLADVAK_APP-pwa";
var filesToCache = ["./", "./index.html", "./BLADVAK_APP.js", "./BLADVAK_APP_bg.wasm"];

async function networkFirst(request) {
    try {
        const networkResponse = await fetch(request);
        if (networkResponse.ok) {
            const cache = await caches.open(cacheName);
            cache.put(request, networkResponse.clone());
        }
        return networkResponse;
    } catch (error) {
        const cachedResponse = await caches.match(request);
        return cachedResponse || Response.error();
    }
}

self.addEventListener("fetch", (event) => {
    event.respondWith(networkFirst(event.request));
});
