/*

https://api.mojang.com/users/profiles/minecraft/[USERNAME] - returns a json with username and ID

https://sessionserver.mojang.com/session/minecraft/profile/[ID] - returns a json with ID, Name, Properties -> 0 -> name & value (Value is JSON encoded with base64)

value from above decoded -> returns a json with some more info, but most importantly, textures -> SKIN -> url

*/

function getJsonCallback(url, callback) {
    var xmlhttp = new XMLHttpRequest();

    xmlhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            var myObj = JSON.parse(this.responseText);
            callback(myObj);
        } else {
            console.log(`${url} request failed`);
        }
    };
    xmlhttp.open("GET", url, true);
    xmlhttp.send();
}

function getSkin(name) {
    console.log(`Get skin ${name}`);
    const url = `https://api.mojang.com/users/profiles/minecraft/${name}`;

    getJsonCallback(url, (response) => {
        let id = response.id;
        const url = `https://sessionserver.mojang.com/session/minecraft/profile/${id}`;
        getJsonCallback(url, (response) => {
            console.log(response.properties);
        });
    });
}