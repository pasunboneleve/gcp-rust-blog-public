if (!window.__hotReloadSocket) {
    const socket = new WebSocket("ws://" + window.location.host + "/ws");
    socket.onmessage = (event) => {
        if (event.data === "reload") {
            window.location.reload();
        }
    };
    window.__hotReloadSocket = socket;
}
