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

    const triggerReload = () => {
        if (window.__hotReloadController.reloadPending) {
            return;
        }
        window.__hotReloadController.reloadPending = true;
        window.location.reload();
    };

    const connect = () => {
        const controller = window.__hotReloadController;

        if (controller.eventSource) {
            controller.eventSource.close();
            controller.eventSource = null;
        }
        if (!devloopEventsUrl) {
            return;
        }

        const source = new EventSource(devloopEventsUrl);
        controller.eventSource = source;

        source.onmessage = (event) => {
            if (event.data === "reload") {
                triggerReload();
            }
        };

        source.onerror = () => {
            source.close();
            controller.eventSource = null;
            if (!controller.reloadPending) {
                window.setTimeout(connect, 500);
            }
        };
    };

    window.addEventListener("beforeunload", () => {
        const controller = window.__hotReloadController;
        controller.reloadPending = true;
        if (controller.eventSource) {
            controller.eventSource.close();
            controller.eventSource = null;
        }
    });

    window.addEventListener("pageshow", () => {
        const controller = window.__hotReloadController;
        if (controller.reloadPending) {
            controller.reloadPending = false;
        }
        if (!controller.eventSource) {
            connect();
        }
    });

    window.__hotReloadController = {
        eventSource: null,
        reloadPending: false,
    };
    reportCurrentPath();
    window.addEventListener("popstate", reportCurrentPath);

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", connect, { once: true });
    } else {
        connect();
    }
}
