let modListJson = [];
let previousToggles = [];

fetch('/mod-list')
    .then(res => res.json())
    .then(json => {
        modListJson = json;

        const list = document.getElementById("olist");
        list.innerHTML = ""; // Clear previous items

        console.log(modListJson);

        if (Object.keys(modListJson).length > 0) {
            for (let i = 0; i < modListJson.length; i++) {
                const item = document.createElement("li");
                const [path_dirty, modName, modType, enabled] = modListJson[i];
                const path = path_dirty.replace("/", "%2F");

                item.innerHTML = `
          <h2>
            <strong>${modName}</strong><br>
            Type: <strong>${modType}</strong><br>
            <label class="toggle-switch">
              <input id="${path}" type="checkbox" />
              <span class="slider"></span>
            </label>
          </h2>`;

                list.appendChild(item);
                document.getElementById(path).checked = enabled;
                var entry = [path, enabled];

                previousToggles.push(entry);
            }
        } else {
            const item = document.createElement("li");
            item.innerHTML = '<strong>No mods found.</strong>';
            list.appendChild(item);
        }
    })
    .catch(err => {
        console.error("Failed to load mod list:", err);
    });

function updateModStatus() {
    console.log(previousToggles);
    for (var i in previousToggles) {
        console.log(previousToggles[i][0]);
        if (previousToggles[i][1] != document.getElementById(previousToggles[i][0]).checked) {
            previousToggles[i][1] = document.getElementById(previousToggles[i][0]).checked;
            console.log("making request to " + previousToggles[i][0]);
            fetch("/toggle-mod/" + previousToggles[i][0]);
        }
    }

    fetch("/set-param/restart");
    alert("Updated preferences.");
}

document.getElementById("submit-mods").onclick = updateModStatus.bind(document.getElementById("submit-mods"));

updateStats();