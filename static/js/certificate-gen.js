fetch('/local-ip')
    .then(response => response.text())
    .then(text => {
        document.getElementById("ip").value = text;
    })
    .catch(error => {
        console.error('Error fetching data:', error);
    });


document.getElementById("caButton").addEventListener("click", function (event) {
    event.preventDefault();

    const caName = document.getElementById("caname").value;
    const caLifetime = document.getElementById("califetime").value;
    const caFileName = document.getElementById("cafilename").value;
    const caKeyName = document.getElementById("cakeyname").value;


    fetch("/generate-ca", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            ca_name: caName,
            ca_lifetime: parseInt(caLifetime),
            ca_file_name: caFileName,
            ca_key_name: caKeyName
        })
    })
        .then(response => {
            if (!response.ok) {
                throw new Error("Network response was not ok");
            }

            return response.json();
        })
        .then(data => {
            alert(data);
        })
        .catch(error => {
            console.error("Error:", error);
            alert("There was an error generating the certificate.");
        });
});

document.getElementById("certButton").addEventListener("click", function (event) {
    event.preventDefault();

    const hostname = document.getElementById("hostname").value;
    const ip = document.getElementById("ip").value;
    const certLifetime = document.getElementById("certlifetime").value;
    const caNameInput = document.getElementById("canameinput").value;
    const caKeyInput = document.getElementById("cakeyinput").value;
    const abCertName = document.getElementById("abcertname").value;
    const abCertKeyName = document.getElementById("abcertkeyname").value;

    const abInfoCertName = document.getElementById("abinfocertname").value;
    const abInfoCertKeyName = document.getElementById("abinfocertkeyname").value;

    fetch("/generate-cert", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            hostname: hostname,
            ip: ip,
            cert_lifetime: parseInt(certLifetime),
            ca_name_input: caNameInput,
            ca_key_input: caKeyInput,
            cert_name: abCertName,
            cert_key_name: abCertKeyName
        })
    })
        .then(response => {
            if (!response.ok) {
                throw new Error("Network response was not ok");
            }
            return response.json();
        })
        .then(data => {
            alert(data);
        })
        .catch(error => {
            console.error("Error:", error);
            alert("There was an error generating the certificate.");
        });

    fetch("/generate-cert", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            hostname: hostname,
            ip: ip,
            cert_lifetime: parseInt(certLifetime),
            ca_name_input: caNameInput,
            ca_key_input: caKeyInput,
            cert_name: abInfoCertName,
            cert_key_name: abInfoCertKeyName
        })
    })
        .then(response => {
            if (!response.ok) {
                throw new Error("Network response was not ok");
            }
            return response.json();
        })
        .then(data => {
            alert(data);
        })
        .catch(error => {
            console.error("Error:", error);
            alert("There was an error generating the certificate.");
        });
});