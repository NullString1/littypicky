export default {
  async fetch(request) {
    const url = new URL(request.url);
    const target = new URL("https://api-littypicky.nullstring.one");
    target.pathname = url.pathname; //.replace(/^\/api/, '') || '/';
    target.search = url.search;
    const proxied = new Request(target.toString(), request);
    return fetch(proxied);
  },
};
