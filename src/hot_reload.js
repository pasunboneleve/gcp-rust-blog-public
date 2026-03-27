if (!window.__hotReloadController) {
    const devloopEventsUrl = __DEVLOOP_BROWSER_EVENTS_URL__;
    const reportCurrentPath = () => {
        fetch("/__dev/current-path", {
            method: "POST",
            headers: { "content-type": "text/plain" },
            body: window.location.pathname,
            keepalive: true,
        }).catch(() => {});
    };

    const connect = () => {
        if (!devloopEventsUrl) {
            return;
        }

        const controller = window.__hotReloadController;
        const source = new EventSource(devloopEventsUrl);
        controller.eventSource = source;

        source.onmessage = (event) => {
            if (event.data === "reload") {
                window.location.reload();
            }
        };

        source.onerror = () => {
            source.close();
            controller.eventSource = null;
            window.setTimeout(connect, 500);
        };
    };

    window.__hotReloadController = {
        eventSource: null,
    };
    reportCurrentPath();
    window.addEventListener("popstate", reportCurrentPath);
    connect();
}
