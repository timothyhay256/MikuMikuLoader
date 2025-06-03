var scenesData = [];
var index = 0;

document.getElementById("add-scene-btn").addEventListener("click", addScene);

function addScene() {

    const scrollmenu = document.getElementById("scrollmenu");

    const firstSceneBlock = scrollmenu.firstElementChild;
    const newSceneBlock = firstSceneBlock.cloneNode(true);
    newSceneBlock.style.display = "";

    const oldIndex = "INDEX";
    const newIndex = index;

    // Recursively update IDs and text
    function updateAttributes(element) {
        if (element.id) {
            element.id = element.id.replace(`${oldIndex}`, `${newIndex}`);
        }
        if (element.htmlFor) {
            element.htmlFor = element.htmlFor.replace(`${oldIndex}`, `${newIndex}`);
        }
        if (element.getAttribute("name")) {
            element.setAttribute("name", element.getAttribute("name").replace(`${oldIndex}`, `${newIndex}`));
        }

        // Update scene button text
        if (element.tagName === 'BUTTON' && element.id && element.id.startsWith('scene-')) {
            element.innerHTML = `Scene ${newIndex + 1}<br>Click to add Sekai Stories JSON`;
        }

        Array.from(element.children).forEach(updateAttributes);
    }

    updateAttributes(newSceneBlock);
    scrollmenu.appendChild(newSceneBlock);

    // Attach modal behavior for the new scene
    const modal = newSceneBlock.querySelector(`#modal-${newIndex}`);
    const btn = newSceneBlock.querySelector(`#scene-${newIndex}`);
    const json = newSceneBlock.querySelector(`#json-${newIndex}`);
    const span = modal.querySelector(".close");

    btn.onclick = function () {
        modal.style.display = "block";
        auto_grow(json);
    }

    span.onclick = function () {
        modal.style.display = "none";
    }

    window.addEventListener("click", function handler(event) {
        if (event.target === modal) {
            modal.style.display = "none";
        }
    });

    document.getElementById(`remove-${newIndex}`).onclick = function (event) {
        event.preventDefault();
        scrollmenu.removeChild(document.getElementById(`scene-container-${newIndex}`));
        scenesData = scenesData.filter(item => item.scene.index !== newIndex);
    }

    document.getElementById(`submit-${newIndex}`).onclick = function (event) {
        event.preventDefault();

        const jsonText = document.getElementById(`json-${newIndex}`).value;

        try {
            const parsed = JSON.parse(jsonText);
            const item = { index: parseInt(newIndex), data: parsed };

            const exists = scenesData.some(existingItem =>
                existingItem.index === item.index &&
                JSON.stringify(existingItem.data) === JSON.stringify(item.data)
            );

            if (!exists) {
                scenesData.push(item);
            } else {
                console.log("scenesData already includes scene, not pushing!");
            }

            alert("Scene saved!");
        } catch (e) {
            alert("Invalid JSON: " + e.message);
        }

        document.getElementById(`modal-${newIndex}`).style.display = "none";
    }

    document.getElementById(`reorder-${newIndex}`).onclick = function (event) {
        event.preventDefault();

        let index = prompt("New index", "0");

        try {
            index = parseInt(index);
            moveElementToIndex(document.getElementById(`scene-container-${newIndex}`), index);

        } catch (error) {
            alert(`Failed to set index: ${error}`)
        }

        document.getElementById(`modal-${newIndex}`).style.display = "none";
    }
    index++;
}

document.getElementById("submit-scenes").addEventListener("click", function (event) {
    event.preventDefault();

    const filename = document.getElementById("storyfile").value;
    const data = JSON.stringify({
        scenesData
    });

    var dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(data);
    var dlAnchorElem = document.getElementById('downloadAnchorElem');
    dlAnchorElem.setAttribute("href", dataStr);
    dlAnchorElem.setAttribute("download", filename);
    dlAnchorElem.click();
});

document.getElementById("generate-mod").addEventListener("click", function (event) {
    event.preventDefault();

    const storyFile = document.getElementById("modpackfile").value;
    const data = JSON.stringify({
        file_name: storyFile,
        data: scenesData
    });

    console.log(`Trying to submit ${data}`);

    fetch("/export-custom-story", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: data
    })
        .then(response => {
            if (!response.ok) {
                throw new Error("Network response was not ok");
            }

            return response.text();
        })
        .then(data => {
            alert(data.text());
        })
        .catch(error => {
            console.error("Error:", error);
            alert(`There was an error exporting the story. Please make sure each scene has valid SEKAI-Stories JSON. Error: ${error}`);
        });
});

function auto_grow(element) {
    element.style.height = "5px";
    element.style.height = (element.scrollHeight) + "px";
}

function moveElementToIndex(element, newIndex) {
    const parent = element.parentNode;
    const children = Array.from(parent.children);
    const currentIndex = children.indexOf(element);

    if (currentIndex === -1) {
        throw new Error("Element not found in parent");
    }

    if (newIndex < 0 || newIndex > children.length - 1) {
        throw new Error("Invalid index");
    }

    parent.removeChild(element);

    if (newIndex > currentIndex) {
        parent.insertBefore(element, children[newIndex]);
    } else {
        parent.insertBefore(element, children[newIndex]);
    }
}

const streamToText = async (blob) => {
    const readableStream = await blob.getReader();
    const chunk = await readableStream.read();

    return new TextDecoder('utf-8').decode(chunk.value);
};

const fileSelector = document.getElementById('file-selector');
fileSelector.addEventListener('change', (event) => {
    let file = event.target.files[0];

    (async () => {
        const fileContent = await file.text();
        var arr = JSON.parse(fileContent);

        const scenes = document.getElementById("scrollmenu");

        if (scenes != null) {
            while (scenes.children.length > 1) {
                scenes.removeChild(scenes.lastElementChild);
            }
        }

        scenesData = [];

        for (var i = 0; i < arr.scenesData.length; i++) {
            index = arr.scenesData[i].index;
            addScene();

            document.getElementById(`json-${index - 1}`).value = JSON.stringify(arr.scenesData[i].data, null, 2);

            scenesData.push({ index: parseInt(index - 1), data: arr.scenesData[i].data });
        }
    })();
});
