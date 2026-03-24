if (!window.__hotReloadController) {
    const protocol = window.location.protocol === "https:" ? "wss://" : "ws://";

    const connect = () => {
        const socket = new WebSocket(protocol + window.location.host + "/ws");
        window.__hotReloadController.socket = socket;

        socket.onmessage = (event) => {
            if (event.data === "reload") {
                window.location.reload();
            }
        };

        socket.onclose = () => {
            window.setTimeout(connect, 500);
        };

        socket.onerror = () => {
            socket.close();
        };
    };

    window.__hotReloadController = { socket: null };
    connect();
}
