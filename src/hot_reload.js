if (!window.__hotReloadController) {
    const devloopEventsUrl = __DEVLOOP_BROWSER_EVENTS_URL__;
    const websocketUrl =
        window.location.protocol === "https:"
            ? `wss://${window.location.host}/ws`
            : `ws://${window.location.host}/ws`;
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
        if (controller.socket && controller.socket.readyState < WebSocket.CLOSING) {
            controller.socket.close();
        }
        controller.socket = null;

        if (devloopEventsUrl) {
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
                window.setTimeout(connect, 500);
            };
        }

        const socket = new WebSocket(websocketUrl);
        controller.socket = socket;

        socket.onmessage = (event) => {
            if (event.data === "reload") {
                triggerReload();
            }
        };

        socket.onclose = () => {
            controller.socket = null;
            if (!controller.reloadPending) {
                window.setTimeout(connect, 500);
            }
        };

        socket.onerror = () => {
            socket.close();
        };
    };

    window.addEventListener("beforeunload", () => {
        const controller = window.__hotReloadController;
        controller.reloadPending = true;
        if (controller.eventSource) {
            controller.eventSource.close();
            controller.eventSource = null;
        }
        if (controller.socket) {
            controller.socket.close();
            controller.socket = null;
        }
    });

    window.addEventListener("pageshow", () => {
        const controller = window.__hotReloadController;
        if (controller.reloadPending) {
            controller.reloadPending = false;
        }
        if (!controller.eventSource && !controller.socket) {
            connect();
        }
    });

    window.__hotReloadController = {
        eventSource: null,
        socket: null,
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
