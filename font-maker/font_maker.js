function notRendered() {
    const ret = new Set();
    for (let i = 0; i <= 31; ++i) {
        ret.add(i);
    }
    ret.add(127);
    for (const value of Object.keys(specialValues())) {
        ret.delete(value);
    }
    return ret;
}

function specialValues() {
    return Object.fromEntries(
        Object.entries({
            "\0": "0",
            "\n": "n",
            "\r": "r",
            "\t": "t",
        }).map(([ch, r]) => [ch.charCodeAt(0), "\\" + r]),
    );
}

class Grapheme {
    static notRendered = notRendered();
    static specialValues = specialValues();

    static graphemeInfo(value) {
        const special = Grapheme.specialValues[value];
        if (special !== undefined) {
            return special;
        }
        const notRendered = Grapheme.notRendered.has(value);
        if (notRendered) {
            const el = document.createElement("grapheme-not-rendered");
            el.textContent = "XX";
            return el;
        }
        return String.fromCharCode(value).padStart(2);
    }
    static stateToHex(state) {
        const hex = new Array(8).fill(0);
        for (let y = 0; y < 8; ++y) {
            let byte = 0;
            for (let x = 0; x < 8; ++x) {
                if (state[y * 8 + x]) {
                    byte |= 1 << 7 - x;
                }
            }
            hex[y] = byte;
        }
        return hex;
    }
    static setStateFromHex(state, hex) {
        for (let y = 0; y < 8; ++y) {
            for (let x = 0; x < 8; ++x) {
                state[y * 8 + x] = (hex[y] >> 7 - x & 1) != 0;
            }
        }
    }

    static setStateFromHexString(state, hex) {
        if (hex.trim() === "") {
            Grapheme.setStateFromHex(state, new Array(8).fill(0));
            return;
        }
        const split = hex.match(/.{1,2}/g);
        const hexValues = split.map((x) => parseInt(x, 16));
        Grapheme.setStateFromHex(state, hexValues);
    }

    static stateToHexString(state) {
        return Grapheme.stateToHex(state)
            .map((v) => v.toString(16).padStart(2, "0"))
            .join("");
    }

    static renderInfo(value) {
        return [
            value.toString().padEnd(3),
            " | ",
            Grapheme.graphemeInfo(value),
        ];
    }

    constructor(value, state = new Array(64).fill(false)) {
        this.value = value;
        this.state = state;
        this.container = document.createElement("grapheme");
        this.canvas = document.createElement("canvas");
        this.canvas.width = 48;
        this.canvas.height = 48;
        const info = document.createElement("info");
        info.append(...Grapheme.renderInfo(value));
        this.container.append(this.canvas, info);
    }

    static renderGrid(canvas) {
        const { width, height } = canvas;
        const g = canvas.getContext("2d");
        g.strokeStyle = "gray";
        g.lineWidth = 2;
        for (let y = 0; y < 8; ++y) {
            for (let x = 0; x < 8; ++x) {
                g.strokeRect(
                    x * (width / 8),
                    y * (height / 8),
                    width / 8,
                    height / 8,
                );
            }
        }
    }
    static render(canvas, state) {
        const { width, height } = canvas;
        const g = canvas.getContext("2d");
        g.imageSmoothingEnabled = false;
        g.fillStyle = "black";
        g.fillRect(0, 0, width, height);
        g.fillStyle = "white";
        for (let y = 0; y < 8; ++y) {
            for (let x = 0; x < 8; ++x) {
                if (state[y * 8 + x]) {
                    g.fillRect(
                        x * (width / 8),
                        y * (height / 8),
                        width / 8,
                        height / 8,
                    );
                }
            }
        }
    }

    deselect() {
        this.container.removeAttribute("active");
    }
    select() {
        this.container.setAttribute("active", "");
    }
}

const overwrite = document.querySelector("#overwrite");
const exportCode = document.querySelector("#export-area");
const interact = document.querySelector("#interact");
const graphemeListElement = document.querySelector("grapheme-list");
const graphemes = [];
let selectedGrapheme;
const selectedGraphemeInfoElement = document.querySelector(
    "selected-grapheme-info",
);
for (let i = 0; i <= 127; ++i) {
    const grapheme = new Grapheme(i);
    graphemeListElement.append(grapheme.container);
    grapheme.container.addEventListener("click", () => {
        selectedGrapheme.deselect();
        selectedGrapheme = grapheme;
        selectedGrapheme.select();
        renderEverything();
    });
    Grapheme.render(grapheme.canvas, grapheme.state);
    graphemes.push(grapheme);
}
selectedGrapheme = graphemes[0];
selectedGrapheme.select();

const [width, height] = [400, 400];

interact.width = width;
interact.height = height;

interact.style.width = width + "px";
interact.style.height = height + "px";

function renderExport() {
    function cond(value) {
        const special = Grapheme.specialValues[value];
        if (special !== undefined) {
            return `'${special}'`;
        }
        const isNotRendered = Grapheme.notRendered.has(value);
        if (isNotRendered) {
            return value.toString();
        }
        return `'${String.fromCharCode(value)}'`;
    }
    exportCode.innerText = graphemes.map((grapheme) => {
        return `case ${cond(grapheme.value).padStart(4)}: return 0x${
            Grapheme.stateToHexString(grapheme.state)
        };
    `.trim();
    }).join("\n");
}

function renderEverything() {
    Grapheme.render(interact, selectedGrapheme.state);
    Grapheme.render(selectedGrapheme.canvas, selectedGrapheme.state);
    Grapheme.renderGrid(interact);
    selectedGraphemeInfoElement.replaceChildren(
        ...Grapheme.renderInfo(selectedGrapheme.value),
        " | 0x",
        Grapheme.stateToHexString(selectedGrapheme.state),
    );
    renderExport();
    overwrite.value = Grapheme.stateToHexString(selectedGrapheme.state);
}

overwrite.addEventListener("input", () => {
    Grapheme.setStateFromHexString(selectedGrapheme.state, overwrite.value);
    renderEverything();
});

interact.addEventListener("click", (ev) => {
    const [x, y] = [ev.offsetX / width * 8, ev.offsetY / height * 8].map(
        Math.floor,
    );
    selectedGrapheme.state[y * 8 + x] = !selectedGrapheme.state[y * 8 + x];
    renderEverything();
});

renderEverything();
