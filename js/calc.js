var operation = "";
var res;
var isOperating = false;

function digitNumber(digit, event) {
    var inputCalc = event.target.parentNode.parentNode.querySelectorAll("#inputCalc")[0];
    if (isOperating === true) {
        inputCalc.value = "";
        operation = "";
        isOperating = false;
    }
    if (inputCalc.value.length < 10) {
        inputCalc.style.fontSize = 35 + "px";
    }

    if (inputCalc.value.length > 10 && inputCalc.value.length < 12) {
        inputCalc.style.fontSize = 35 + "px";
    } else if (inputCalc.value.length > 17 && inputCalc.value.length < 25) {
        inputCalc.style.fontSize = 20 + "px";
    } else if (inputCalc.value.length > 25)
        inputCalc.style.fontSize = 12 + "px";

    inputCalc.value += digit;
}

function digitOperation(op, event) {
    var inputCalc = event.target.parentNode.parentNode.querySelectorAll("#inputCalc")[0];
    if (isOperating === true) {
        inputCalc.value = res;
        isOperating = false;
        operation = "";
    }

    if (inputCalc.value.length > 10 && inputCalc.value.length < 12) {
        inputCalc.style.fontSize = 35 + "px";
    } else if (inputCalc.value.length > 12) {
        inputCalc.style.fontSize = 20 + "px";
    }

    if (operation != "") {
        inputCalc.value = inputCalc.value.replace(operation, op);
        operation = op;
    } else {
        inputCalc.value += " " + op + " ";
        operation = op;
    }
}

function canc(e) {
    var inputCalc = e.target.parentNode.parentNode.querySelectorAll("#inputCalc")[0];
    var outputCalc = e.target.parentNode.parentNode.querySelectorAll("#outCalc")[0];
    if (inputCalc.value.length < 10) {
        inputCalc.style.fontSize = 35 + "px";
    }
    operation = "";
    inputCalc.value = "";
    outputCalc.value = "";
}

var sum = (a, b) => {
    return a + b
}
var prod = (a, b) => {
    return a * b
}
var diff = (a, b) => {
    return a - b
}
var div = (a, b) => {
    return a / b
}

function isOperation(input) {
    if (input === "+")
        return sum;
    else if (input === "-")
        return diff;
    else if (input === "x")
        return prod;
    else if (input === ":") {
        return div;
    } else
        return false;
}

function calcolatrice(a, b, op) {
    return op(a, b);
}

function isInt(n) {
    return n % 1 === 0;
}

function getRes(e) {
    var inputCalc = e.target.parentNode.parentNode.querySelectorAll("#inputCalc")[0];
    var display = e.target.parentNode.parentNode.querySelectorAll("#displayCalc")[0];
    var outputCalc = e.target.parentNode.parentNode.querySelectorAll("#outCalc")[0];

    if (isOperating === true) {
        inputCalc.value = res;
        isOperating = false;
        operation = "";
        return;
    }
    let input = inputCalc.value.split(" ");



    if (input.length != 3) {
        outputCalc.value = "ERROR";
    } else {
        let a, b;

        a = input[0];
        b = input[2];

        if (a.includes(".") || b.includes(".")) {
            a = parseFloat(a);
            b = parseFloat(b);
        } else {
            a = parseInt(a);
            b = parseInt(b);
        }


        res = calcolatrice(a, b, isOperation(operation));
        if (!isInt(res)) {
            res = res.toFixed(3);
        }

        let str_res = "= " + res;
        if (str_res.length > 15) {
            outputCalc.style.fontSize = 55 + "px";
        }
        outputCalc.value = str_res;

    }

    isOperating = true;
    display.appendChild(outputCalc);
}