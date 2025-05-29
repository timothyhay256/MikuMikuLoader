async function updateStats() {
    const totalPassthrough = await fetch('/total-passthrough');
    const totalProxied = await fetch('/total-proxied');
    const totalRequests = await fetch('/total-requests');
    const totalRequestsJson = await totalRequests.json();

    document.getElementById('total-passthrough').innerHTML = `<p class=serverStatus>Total requests passed-through to origin: ${await totalPassthrough.text()}</p>`;
    document.getElementById('total-proxied').innerHTML = `<p class=serverStatus>Total requests proxied: ${await totalProxied.text()}</p>`;

    var list = document.getElementById("olist");

    list.innerHTML = ""; // Clear previous items

    for (let i of totalRequestsJson) {
        const item = document.createElement("li");
        var [status, path, injected_path] = i;

        if (injected_path == null) {
            injected_path = "No file injected";
        }

        item.innerHTML = `<strong>${status}</strong>: <code>${path}</code> <strong><-</strong> <code>${injected_path}</code>`;
        list.appendChild(item);
    }
}

function sendCommand(param) {
    fetch("/set-param/" + param);
}

document.getElementById("start-injection").onclick = sendCommand.bind(document.getElementById("start-injection"), "start");
document.getElementById("stop-injection").onclick = sendCommand.bind(document.getElementById("stop-injection"), "stop");
setInterval(updateStats, 500);