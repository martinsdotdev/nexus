// The login page is a client-rendered form; prerender the static shell like the
// other routes. The /auth/* calls run in the browser against the cloud-mode relay.
export const prerender = true;
