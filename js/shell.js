var sum = (a, b) => { return a + b }
var prod = (a, b) => { return a * b }
var diff = (a, b) => { return a - b }
var div = (a, b) => { return a / b }

function calcolatrice(a, b, op) {
    return op(a, b);
}

var max_temp = 50;
var min_temp = 0;

var user = "cicciodibbi@ubuntu:~$";

const news_sidebar = document.querySelectorAll(".news-container")[0];

const shell_div = document.getElementById("container");

var recent_cmd = [];

var recent_index = 0;

var shell_id = 0;

var last_cmd_length = 0;

const shell_btn = document.getElementById('shell-btn');

const temp_btn = document.getElementById('temp-btn');

const temp_div = Array.from(document.getElementsByClassName("weather-cont"))[0];

const calcs_div = Array.from(document.getElementsByClassName("calcolatrice"))[0];

const desktop = document.getElementById('desktop');

var packets = 0;

var shell_count = 0;

var temp_count = 0;

const footer = document.querySelector("footer");

const side_infos = footer.querySelector("#side-infos");
const side_date = side_infos.querySelector("#side-date");
const side_time = side_infos.querySelector("#side-time");

var hiddens_shells = new Array();

var hiddens_shells_index = 0;

var hiddens_temps = new Array();

var hiddens_temps_index = 0;

var hiddens_calcs = new Array();

var hiddens_calcs_index = 0;

var isOver = false;

var hiddens_timers = new Array();

var hiddens_timers_index = 0;

const timer_div = document.getElementById("timer");

var funn = (s) => {
    for (let i = 0; i < 10; i++) {
        alert(s);
    }
}

function fakePing(site, shell, index) {

    var delayInMilliseconds = 500; //1 second

    setTimeout(function() {
        //your code to be executed after 1 second
        if (index == packets + 1) {
            let str = "\nPING " + site + " (142.251.209.14) 56(84) bytes of data.\n";
            shell.value += str;
        } else if (index == 1) {
            str = "64 bytes from mil04s50-in-f14.1e100.net (142.251.209.14): icmp_seq=" + (packets + 1 - index) + " ttl=113 time=" + (Math.random() * (60.1 - 12.5 + 1) + 12.5) + " ms\n";
            shell.value += str;
            return "ping";
        } else {
            str = "64 bytes from mil04s50-in-f14.1e100.net (142.251.209.14): icmp_seq=" + (packets + 1 - index) + " ttl=113 time=" + (Math.random() * (60.1 - 12.5 + 1) + 12.5) + " ms\n";
            shell.value += str;
        }

        return fakePing(site, shell, index - 1);
    }, delayInMilliseconds);

}

function getOperation(value) {
    var lines = value.split("\n");
    var arguments = lines[lines.length - 1].split("$");

    var op = arguments[1].substring(1, arguments[1].length);
    return op;
}

function getContent(value) {
    var lines = value.split("\n");
    //if (lines.length > 1)
    lines.pop();
    let str = "";
    for (let i = 0; i < lines.length; i++) {
        str += lines[i];
        if (i < lines.length - 1) {
            str += "\n";
        }
    }
    return str;
}

function computeCommand(value, shell) {
    let response = [];
    console.log(value);

    var lines = value.split("\n");
    console.log(lines[lines.lenght - 1]);
    var arguments = lines[lines.length - 1].split(" ");
    console.log("op" + arguments[1]);
    console.log("n1" + arguments[2]);
    console.log("n2" + arguments[3]);

    //arguments[0] = user , arguments[1] = command, arguments[2] = n1, arguments[3] = n2
    var op = arguments[1];
    var n1 = (arguments[2]);
    var n2 = (arguments[3]);

    switch (op) {
        case "cmd":
            value = "Lista comandi:\nsum <n1> <n2>\ndiff <n1> <n2>\nprod <n1> <n2>\ndiv <n1> <n2>\ntime\nfun\nmorefun\nch-color <color>\nch-nav-color <color>\nch-back-color <color>\nch-txt-color <color>\nfont-size <size>\nping <server>\nopen <URL>\nclear";
            break;
        case "sum":
            value = calcolatrice(parseInt(n1), parseInt(n2), sum);
            break;
        case "diff":
            value = calcolatrice(parseInt(n1), parseInt(n2), diff);
            break;
        case "prod":
            value = calcolatrice(parseInt(n1), parseInt(n2), prod);
            break;
        case "div":
            value = calcolatrice(parseInt(n1), parseInt(n2), div);
            break;
        case "time":
            value = new Date().toLocaleString();
            break;
        case "fun":
            value = "";
            funn("FUN IS HERE")
            break;
        case "morefun":
            value = "";
            funn("\ _________________________\n....Moooore Fun\n-------------------------\n      o   ^__^\n       o  (oo)\_______\n            (__)\       )\/\ \n              ||----w |\n              ||         ||")
            break;
        case "clear":
            value = "clear";
            break;
        case "calc":
            value = "";
            createCalcsWindow();
            break;
        case "temp":
            value = "";
            createTempWindow();
            break;
        case "terminal":
            value = "";
            createShellWindow();
            break;
        case "open":
            value = "";
            window.open(n1, '_blank');
            break;
        case "back-img":
            value = "";
            document.body.style.backgroundImage = "url('./" + n1 + "')";
            break;
        case "font-size":
            value = "";
            shell.style.fontSize = n1;
            break;
        case "ch-color":
            shell.style.backgroundColor = n1;
            value = "";
        case "ch-nav-color":
            var nav = document.getElementById("shell-bar");
            var container = document.getElementById("container");
            container.style.backgroundColor = n1;
            container.style.borderColor = n1;
            nav.style.backgroundColor = n1;
            value = "";
            break;
        case "ch-back-color":
            shell.style.backgroundColor = n1;
            value = "";
            break
        case "ch-txt-color":
            shell.style.color = n1;
            value = "";
            break
        case "ping":
            packets = parseInt(n2);
            fakePing(n1, shell, packets + 1);
            value = "ping"
            break;
        default:
            return ["", ""];
    }
    if (n1 == undefined && n2 == undefined)
        response[0] = op;
    else if (n1 != undefined && n2 == undefined)
        response[0] = op + " " + n1;
    else
        response[0] = op + " " + n1 + " " + n2;

    response[1] = value;
    last_cmd_length = response[0].length;
    return response
}

function getRandomInt(min, max) {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

function createShellWindow() {
    let clonedShell = shell_div.cloneNode(true);

    clonedShell.style.top = 0 + "px";
    clonedShell.style.left = 0 + "px";
    clonedShell.style.visibility = "visible";
    if (clonedShell.classList.contains("animated-hidden")) {
        clonedShell.classList.toggle("animated-hidden");
    }
    if (clonedShell.classList.contains("animated-close")) {
        clonedShell.classList.toggle("animated-close");
    }
    clonedShell.style.zIndex = zindexes++;
    desktop.appendChild(clonedShell);
    //clonedShell.addEventListener('mousedown', mouseDownHandler);
    console.log(clonedShell);
    move();
}

function createCalcsWindow() {
    let clonedCalcs = calcs_div.cloneNode("deep");

    clonedCalcs.style.top = 0 + "px";
    clonedCalcs.style.left = 0 + "px";
    clonedCalcs.style.visibility = "visible";
    if (clonedCalcs.classList.contains("animated-hidden")) {
        clonedCalcs.classList.toggle("animated-hidden");
    }
    if (clonedCalcs.classList.contains("animated-close")) {
        clonedCalcs.classList.toggle("animated-close");
    }


    clonedCalcs.style.zIndex = zindexes++;
    desktop.appendChild(clonedCalcs);
    console.log(clonedCalcs.querySelector("#inputCalc"));
    clonedCalcs.querySelector("#inputCalc").value = "";
    clonedCalcs.querySelector("#outCalc").value = "";
    //clonedCalcs.addEventListener('mousedown', mouseDownHandler);
    console.log(clonedCalcs);
    move();
}

function createTempWindow() {
    let clonedTemp = temp_div.cloneNode("deep");

    clonedTemp.style.top = 0 + "px";
    clonedTemp.style.left = 0 + "px";
    clonedTemp.style.visibility = "visible";
    clonedTemp.classList.remove("animated-hidden");
    clonedTemp.classList.remove("animated-close");
    clonedTemp.style.zIndex = zindexes++;

    fetchWeatherData(clonedTemp, "Catania")

    onresize(clonedTemp, function() {
        console.log("we")
        let city_name = Array.from(clonedTemp.getElementsByClassName("city-name"))[0];
        let state_name = Array.from(clonedTemp.getElementsByClassName("state-name"))[0];
        let temp = Array.from(clonedTemp.getElementsByClassName("temp"))[0];
        let humid = Array.from(clonedTemp.getElementsByClassName("humidity"))[0];
        let infos = Array.from(clonedTemp.getElementsByClassName("infos"))[0];


        let y = parseInt(clonedTemp.style.height.replace("px", ""));
        let x = parseInt(clonedTemp.style.width.replace("px", ""))

        console.log("x e y: ", x + " " + y);
        city_name.style.fontSize = Math.min(x * 7 / 100, y * 7 / 100) + "px";
        state_name.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";
        temp.style.fontSize = Math.min(x * 12 / 100, y * 12 / 100) + "px";
        humid.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";
        infos.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";

    });
    //clonedTemp.addEventListener('mousedown', mouseDownHandler);
    desktop.appendChild(clonedTemp);
    console.log(clonedTemp);
    move();
}

function createTimerWindow() {
    let clonedTimer = timer_div.cloneNode("deep");

    clonedTimer.style.top = 0 + "px";
    clonedTimer.style.left = 0 + "px";
    clonedTimer.style.visibility = "visible";
    clonedTimer.classList.remove("animated-hidden");
    clonedTimer.classList.remove("animated-close");
    clonedTimer.style.zIndex = zindexes++;
    clonedTimer.querySelector(".timer-button").id = ++timer_index;
    clonedTimer.querySelector(".timer-del-button").id = timer_index;
    //clonedTemp.addEventListener('mousedown', mouseDownHandler);
    desktop.appendChild(clonedTimer);
    move();
}

function searchCity(e) {
    console.log(e.target.parentNode.parentNode.querySelectorAll("#search-input"));
    let cityName = e.target.parentNode.parentNode.querySelectorAll("#search-input")[0].value;
    fetchWeatherData(e.target.parentNode.parentNode.parentNode, cityName);
}

var dragValue;

var zindexes = 0;

var pos1 = 0,
    pos2 = 0,
    pos3 = 0,
    pos4 = 0;

function move(id) {

    var bars = document.getElementsByClassName("windows-bar");
    Array.from(bars).forEach((bar) => {
        bar.onmousedown = function(e) {
            dragValue = bar.parentNode;
            dragValue.style.zIndex = 9999;

            e = e || window.event;
            e.preventDefault();
            // get the mouse cursor position at startup:
            pos3 = e.clientX;
            pos4 = e.clientY;
        }
    })
}

function halfScreen(window, pos) {
    window.style.width = "50%";
    window.style.height = "calc(100% - 50px)";

    if (pos == "left") {
        window.style.right = null;
        window.style.left = 0;
        window.style.top = 0;
    } else if (pos == "right") {
        window.style.left = null;
        window.style.right = 0;
        window.style.top = 0;
    } else if (pos == "left-t") {
        window.style.right = null;
        window.style.left = 0;
        window.style.top = 0;
        window.style.width = "50%";
        window.style.height = "50%";
    } else if (pos == "left-b") {
        window.style.right = null;
        window.style.left = 0;
        window.style.top = "calc(50%";
        window.style.width = "50%";
        window.style.height = "calc(50% - 50px)";
    } else if (pos == "right-t") {
        window.style.left = null;
        window.style.right = 0;
        window.style.top = 0;
        window.style.width = "50%";
        window.style.height = "calc(50%)";
    } else if (pos == "right-b") {
        window.style.left = null;
        window.style.right = 0;
        window.style.top = "calc(50%)";
        window.style.width = "50%";
        window.style.height = "calc(50% - 50px)";
    }

    if (window.className.includes("weather-cont")) {
        changeFontSize(window);
    }
}

document.onmousemove = function(e) {
    if (dragValue != null) {
        e = e || window.event;
        e.preventDefault();
        // calculate the new cursor position:
        pos1 = pos3 - e.clientX;
        pos2 = pos4 - e.clientY;
        pos3 = e.clientX;
        pos4 = e.clientY;
        // set the element's new position:
        dragValue.style.top = (dragValue.offsetTop - pos2) + "px";
        dragValue.style.left = (dragValue.offsetLeft - pos1) + "px";
    }

}

document.onmouseup = function(e) {
    if (dragValue != null) {
        let x = e.clientX;
        let y = e.clientY;

        let h = document.body.offsetHeight - 50;
        let w = document.body.offsetWidth;

        if (x <= 0) {
            if ((y <= h - (h * 10 / 100) && y >= ((h - 50) * 10 / 100)))
                halfScreen(dragValue, "left");
            else if (y >= 0 && y <= (h * 10 / 100))
                halfScreen(dragValue, "left-t");
            else
                halfScreen(dragValue, "left-b")
        } else if (x >= document.body.offsetWidth - 20) {
            if ((y <= h - (h * 10 / 100) && y >= ((h - 50) * 10 / 100)))
                halfScreen(dragValue, "right");
            else if (y >= 0 && y <= (h * 10 / 100))
                halfScreen(dragValue, "right-t");
            else
                halfScreen(dragValue, "right-b")
        } else {
            if (y <= 10) {
                maximiseWindow(dragValue, 'f');
                return;
            }
        }

        dragValue.style.zIndex = zindexes++;
        dragValue = null;

    }

}


desktop.addEventListener("keydown", (event) => {
    var code = event.key;
    var n_code = event.key;
    let value = event.target.value;
    console.log(event.target.value);
    if (code == "Enter") { //Enter keycode
        let response = computeCommand(value, event.target);
        recent_cmd.unshift(response[0]);
        recent_index = 0;
        if (response[1] == "")
            event.target.value += "\n" + user + " ";
        else if (response[1] == "clear") {
            event.target.value = user + " ";
        } else if (response[1] == "ping") {
            var delayInMilliseconds = 550 * (packets + 1); //1 second

            setTimeout(function() {
                event.target.value += "\n" + user + " ";
                console.log("ping");
            }, delayInMilliseconds);
        } else
            event.target.value += "\n" + response[1] + "\n" + user + " ";
    } else if (code == "ArrowDown") {
        console.log("down");
        event.target.value += "\n" + user + " " + recent_cmd[0];
    }

})

function deleteWindow(e) {
    e.target.parentNode.parentNode.classList.add("animated-close");
    setTimeout(() => {
        e.target.parentNode.parentNode.remove();
    }, 200)

}


function createIcon(class1, class2, hidden_array, hidden_index, type) {
    let new_icon = document.createElement("a");
    new_icon.classList.add("footer-icon");
    new_icon.classList.add(type);
    new_icon.href = "#";

    if (type === "news") {
        new_icon.addEventListener("click", (e) => {
            //aprire le news
        })
    } else {
        new_icon.addEventListener("click", (e) => {
            if (hidden_array.length == 0) {
                if (new_icon.className.includes("weather")) {
                    console.log("weather");
                    createTempWindow();
                } else if (new_icon.className.includes("shell")) {
                    createShellWindow();
                    console.log("shell");
                } else if (new_icon.className.includes("calcs")) {
                    createCalcsWindow();
                    console.log("calcs");
                } else if (new_icon.className.includes("timer")) {
                    createTimerWindow();
                    console.log("timer");
                }
            } else {
                hidden_array.forEach(hidden => {
                    hidden.classList.remove("animated-hidden");
                    hidden.classList.add("animated-fadeIn");
                    hidden.style.visibility = "visible";

                    setTimeout(() => {
                        hidden.classList.remove("animated-fadeIn");
                    }, 500);

                });

                let elements = new_icon.getElementsByClassName("window-containers");
                Array.from(elements).forEach(element => {
                    element.remove();
                })

                hidden_array.length = 0;
                hidden_index = 0;

            }

            hidden_array.pop();
        })
    }


    let element_icon = document.createElement("i");
    element_icon.classList.add(class1);
    element_icon.classList.add(class2);

    if (type == "news") {
        new_icon.addEventListener("mouseover", (e) => {
            if (e.target === new_icon)
                toggleAnimationNews();
        })

        footer.appendChild(new_icon);
        new_icon.appendChild(element_icon);
        return;
    }
    let element_reel = document.createElement("div");
    element_reel.classList.add("shell-icon"); //cambiare nome alla classe CSS

    new_icon.addEventListener("mouseover", () => {
        hidden_array.forEach(hidden => {
            let new_div = document.createElement("div");
            let clonedElement = hidden.cloneNode(true);


            new_div.classList.add("window-containers");
            hidden.classList.remove("animated-hidden");
            hidden.style.visibility = "hidden";
            clonedElement.style.visibility = "visible";

            element_reel.appendChild(new_div);
            new_div.appendChild(clonedElement);
            clonedElement.style.visibility = "visible";
            /*
            let offsetX = (310 / clonedShell.offsetWidth) * (-80);
            let offsetY = (clonedShell.offsetHeight / 243.33) * (-80);
            console.log("offsetX e Y " + offsetX + " " + offsetY);*/
            clonedElement.style.width = null;
            clonedElement.style.height = null;
            clonedElement.style.top = 0 + "px";
            clonedElement.style.left = 0 + "px";
            clonedElement.style.position = "relative";
            let resX = (310 / clonedElement.offsetWidth) * 0.3;
            let resY = (243.33 / clonedElement.offsetHeight) * 0.3;
            console.log("width & height: " + clonedElement.offsetWidth + " " + clonedElement.offsetHeight);
            console.log("resX e Y " + resX + " " + resY);
            clonedElement.style.transform = "scale(" + resX + "," + resY + ")";
        })
    })

    new_icon.addEventListener("mouseout", () => {
        let elements = new_icon.getElementsByClassName("window-containers");
        Array.from(elements).forEach(element => {
            element.remove();
        })
    })

    footer.appendChild(new_icon);
    new_icon.appendChild(element_icon);
    new_icon.appendChild(element_reel);
}

function toggleAnimationNews() {
    if (isOver) {
        if (news_sidebar.classList.contains("animated-in")) {
            news_sidebar.classList.toggle("animated-in");
        }
        news_sidebar.classList.toggle("animated-out");
        isOver = false;
    } else {
        if (news_sidebar.classList.contains("animated-out")) {
            news_sidebar.classList.toggle("animated-out");
        }
        news_sidebar.classList.toggle("animated-in");
        isOver = true;
    }
}

createIcon("bx", "bx-terminal", hiddens_shells, hiddens_shells_index, "shell");
createIcon("bx", "bx-sun", hiddens_temps, hiddens_temps_index, "weather");
createIcon("bx", "bx-calculator", hiddens_calcs, hiddens_calcs_index, "calcs");
createIcon("bx", "bx-news", null, null, "news");
createIcon("bx", "bx-timer", hiddens_timers, hiddens_timers_index, "timer");

function hiddenWindow(e) {

    if (e.target.parentNode.parentNode.className.includes("shell-container")) {
        hiddens_shells[hiddens_shells_index++] = e.target.parentNode.parentNode;
        e.target.parentNode.parentNode.classList.add("animated-hidden");
    } else if (e.target.parentNode.parentNode.className.includes("weather-cont")) {
        hiddens_temps[hiddens_temps_index++] = e.target.parentNode.parentNode;
        e.target.parentNode.parentNode.classList.add("animated-hidden");
    } else if (e.target.parentNode.parentNode.className.includes("calcolatrice")) {
        hiddens_calcs[hiddens_calcs_index++] = e.target.parentNode.parentNode;
        e.target.parentNode.parentNode.classList.add("animated-hidden");
    } else {
        hiddens_timers[hiddens_timers_index++] = e.target.parentNode.parentNode;
        e.target.parentNode.parentNode.classList.add("animated-hidden");
    }
}

function maximiseWindow(e, mode) {

    console.log("max");

    let el;
    if (mode == 'b') {
        el = e.target.parentNode.parentNode;
    } else {
        el = e;
    }

    console.log(el);

    if (el.className.includes("weather-cont") || el.className.includes("shell-container") || el.className.includes("calcolatrice") || el.className.includes("timer-cont")) {
        console.log("includes");
        if (el.className.includes("maximised") === true) {
            el.classList.remove("maximised");
            el.style.top = null;
            el.style.left = null;
            el.style.width = null;
            el.style.height = null;
            return;
        } else {
            el.classList.add("maximised");
            el.style.top = 0 + "px";
            el.style.left = 0 + "px";
            el.style.width = document.body.clientWidth + "px";
            el.style.height = document.body.clientHeight - 50 + "px";
        }


        el.style.zIndex = 99999;
        if (el.className.includes("weather-cont") === true) {
            console.log("sono dentro");
            changeFontSize(el);
        } else if (el.className.includes("shell-container")) {
            //e.target.parentNode.parentNode.getElementsByClassName("text-area-shell")[0].style.width = document.body.clientWidth + "px";
            //e.target.parentNode.parentNode.getElementsByClassName("text-area-shell")[0].style.height = "100%";
        } else if (el.className.includes("calcolatrice")) {
            //nothing
        }

        if (el === dragValue) {
            dragValue.style.zIndex = zindexes++;
            dragValue = null;
        }

        return;
    }
    console.log("no includes");
}

const onresize = (element, callback) => {
    const resizeObserver = new ResizeObserver(() => callback());
    resizeObserver.observe(element);
}

window.addEventListener("resize", (event) => {
    console.log(event.target);
})

function changeFontSize(clonedTemp) {
    setTimeout(() => {
        let city_name = Array.from(clonedTemp.getElementsByClassName("city-name"))[0];
        let state_name = Array.from(clonedTemp.getElementsByClassName("state-name"))[0];
        let temp = Array.from(clonedTemp.getElementsByClassName("temp"))[0];
        let humid = Array.from(clonedTemp.getElementsByClassName("humidity"))[0];
        let infos = Array.from(clonedTemp.getElementsByClassName("infos"))[0];

        let y = parseInt(clonedTemp.style.height.replace("px", ""));
        let x = parseInt(clonedTemp.style.width.replace("px", ""))

        city_name.style.fontSize = Math.min(x * 7 / 100, y * 7 / 100) + "px";
        state_name.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";
        temp.style.fontSize = Math.min(x * 12 / 100, y * 12 / 100) + "px";
        humid.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";
        infos.style.fontSize = Math.min(x * 5 / 100, y * 5 / 100) + "px";
    }, 20);

}

document.addEventListener("mousedown", (e) => {
    if (!news_sidebar.contains(e.target)) {
        if (isOver)
            toggleAnimationNews();
    }
})

function fetchWeatherData(window, cityName) {
    fetch('http://api.weatherapi.com/v1/current.json?key=aef20d8493d94896a82134458222905&q=' + cityName + '&aqi=no&lang=it')
        .then(response => response.json())
        .then((data) => {
            console.log(data);
            var { name } = data.location;
            var { text } = data.current.condition;
            var { temp_c, humidity } = data.current;
            var { country } = data.location;
            window.getElementsByClassName("city-name")[0].innerHTML = name;
            window.getElementsByClassName("state-name")[0].innerHTML = country;
            window.getElementsByClassName("temp")[0].innerHTML = temp_c + "°C";
            window.getElementsByClassName("infos")[0].innerHTML = text;
            window.getElementsByClassName("humidity")[0].innerHTML = "Umidità: " + humidity + "%";
        })
        .then(() => {
            fetchBackGroundTemp(window, cityName);
        })
        .catch((error) => {
            console.log(error);
        })
}

function fetchBackGroundTemp(window, city) {
    fetch("https://api.unsplash.com/search/photos?query=" + city + "&client_id=2BHRf_91BeuRTt7CDCE-_eA3l95wlZLWlyog-KQ2c2Y")
        .then(
            function(response) {
                if (response.status !== 200) {
                    console.log("There was a problem. Status code: " + response.status);
                    return;
                }
                var index = getRandomInt(0, 6);
                response.json().then(
                    function(data) {
                        window.querySelector("#temp-background").src = data["results"][index]["urls"]["regular"];
                    }
                )
            }
        )
        .catch(
            function(err) {
                console.log(err + '404');
            }
        )
}

function randomColorWithAlpha(alpha) {
    let r = getRandomInt(0, 255);
    let g = getRandomInt(0, 255);
    let b = getRandomInt(0, 255);

    return "rgba(" + r + "," + g + "," + b + "," + alpha + ")";
}

function changeNewsBackColor() {
    let cards = news_sidebar.getElementsByClassName("card");
    Array.from(cards).forEach(card => {
        card.style.backgroundColor = randomColorWithAlpha(0.8)
    })
}

function changeAllTimes(time) {
    changeNewsTime();
    changeSideTime();
    changeSideDate();

    setInterval(() => {
        changeNewsTime();
        changeSideTime();
        changeSideDate();
    }, time);
}

const news_time = news_sidebar.querySelector(".time-bar")

function changeNewsTime() {
    news_time.innerHTML = getTime();
}

function changeSideTime() {
    side_time.innerHTML = getTime();
}

function changeSideDate() {
    side_date.innerHTML = getDate();
}

changeAllTimes(1000);

function getTime() {
    const date = new Date();
    let hours = date.getHours();
    let minutes = date.getMinutes();

    hours = hours < 10 ? "0" + hours : hours;
    minutes = minutes < 10 ? "0" + minutes : minutes;

    let time = hours + ":" + minutes;

    return time;
}

function getDate() {
    const date = new Date();
    let day = date.getDate();
    let month = date.getMonth() + 1;
    let year = date.getFullYear();
    if (day < 10)
        day = "0" + day;
    if (month < 10)
        month = "0" + month;
    let full_date = day + "/" + month + "/" + year;
    return full_date;
}

changeNewsBackColor()


const battery_icons_div = side_infos.querySelector(".battery-icon");
const battery_icon = battery_icons_div.querySelector(".bxs-battery-full");

// Get the battery API
function getBatteryPerc() {
    navigator.getBattery()
        .then(function(battery) {
            return parseInt(battery.level * 100);
        })
        .then(function(perc) {
            battery_perc.innerHTML = perc + "%";
            if (perc <= 50) {
                battery_icon.classList.remove("bxs-battery-full");
                battery_icon.classList.add("bxs-battery-low");
            } else {
                battery_icon.classList.add("bxs-battery-full");
                battery_icon.classList.remove("bxs-battery-low");
            }

            if (batteryIsCharging) {
                battery_icon.classList.remove("bxs-battery-full");
                battery_icon.classList.remove("bxs-battery-low");
                battery_icon.classList.add("bxs-battery-charging");
            } else {
                battery_icon.classList.remove("bxs-battery-charging");
            }
        });
}

const battery_perc = side_infos.querySelector(".battery-perc");

function changeBatteryPerc(time) {
    getBatteryPerc();
    setInterval(() => {
        getBatteryPerc();
    }, time);
}

let batteryIsCharging = false;

navigator.getBattery().then(function(battery) {
    batteryIsCharging = battery.charging;

    battery.addEventListener('chargingchange', function() {
        batteryIsCharging = battery.charging;
    });
});

changeBatteryPerc(1000);


var progress_bar = document.querySelector(".out");
var number = document.querySelector(".number");


var minutes_in = new Array();
var seconds_in = new Array();

var timer_index = 0;

var progress_end_value = 100;
var speed = 1000;

var tot = new Array();

var isOvertTimer = new Array();
var sec = new Array();
var min = new Array();


function startTimer(e) {
    let getTotal = new Promise(function(resolve, reject) {
        minutes_in[e.target.id] = e.target.parentNode.querySelector(".minutes");
        seconds_in[e.target.id] = e.target.parentNode.querySelector(".seconds");

        let minutes_val = minutes_in[e.target.id].value;
        let seconds_val = seconds_in[e.target.id].value;

        sec[e.target.id] = seconds_val * 1000;
        min[e.target.id] = minutes_val * 1000 * 60;

        let total = min[e.target.id] + sec[e.target.id];
        console.log(total)

        isOvertTimer[e.target.id] = false;

        resolve(total)
    })

    e.target.parentNode.parentNode.style.background = "none";
    getTotal.then((d) => {
        tot[e.target.id] = d;
    }).then(() => timerCircle(e))
}

function toggleIsOverTimer(e) {
    if (isOvertTimer[e.target.id] === undefined) {
        return;
    }
    console.log("ciii")
    isOvertTimer[parseInt(e.target.id)] = true;
    console.log(isOvertTimer[parseInt(e.target.id)])
}

function timerCircle(e) {
    var progress_value = 0;
    let progress = setInterval(() => {
            let indexTimer = parseInt(e.target.id);

            progress_value += progress_end_value / (tot[indexTimer] / speed);
            e.target.parentNode.parentNode.style.background = "conic-gradient( #4d5bf9 " + (progress_value * 3.6) + "deg, " + "#cadcff " + (progress_value * 3.6) + "deg)";

            if (sec[indexTimer] <= 0) {
                seconds_in[indexTimer].value = 0;
                if (min[indexTimer] >= 0) {
                    min[indexTimer] -= speed;
                    let ref = Math.floor(min[indexTimer] / (1000));

                    minutes_in[indexTimer].value = Math.floor((min[indexTimer] / speed) / 60);
                    seconds_in[indexTimer].value = (ref % 60);
                } else {
                    return;
                }
            } else {
                sec[indexTimer] -= speed;
                let ref = Math.floor(sec[indexTimer] / 1000);
                if (ref <= 0) {
                    seconds_in[indexTimer].value = 0;
                } else
                    seconds_in[indexTimer].value = ref;
            }

            if (progress_value >= progress_end_value || isOver[indexTimer] == true) {
                clearInterval(progress);
                return;
            }
            console.log(isOvertTimer[indexTimer])
        },
        speed);
}