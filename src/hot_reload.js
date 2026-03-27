if (!window.__hotReloadController) {
    const protocol = window.location.protocol === "https:" ? "wss://" : "ws://";
    const reportCurrentPath = () => {
        fetch("/__dev/current-path", {
            method: "POST",
            headers: { "content-type": "text/plain" },
            body: window.location.pathname,
            keepalive: true,
        }).catch(() => {});
    };

    const connect = () => {
        const controller = window.__hotReloadController;
        const socket = new WebSocket(protocol + window.location.host + "/ws");
        controller.socket = socket;

        socket.onopen = () => {
            const shouldReload = controller.hasConnected && controller.reloadOnReconnect;
            controller.hasConnected = true;
            if (shouldReload) {
                controller.reloadOnReconnect = false;
                window.location.reload();
            }
        };

        socket.onmessage = (event) => {
            if (event.data === "reload") {
                controller.reloadOnReconnect = false;
                window.location.reload();
            }
        };

        socket.onclose = () => {
            if (controller.hasConnected) {
                controller.reloadOnReconnect = true;
            }
            controller.socket = null;
            window.setTimeout(connect, 500);
        };

        socket.onerror = () => {
            socket.close();
        };
    };

    window.__hotReloadController = {
        socket: null,
        hasConnected: false,
        reloadOnReconnect: false,
    };
    reportCurrentPath();
    window.addEventListener("popstate", reportCurrentPath);
    connect();
}
