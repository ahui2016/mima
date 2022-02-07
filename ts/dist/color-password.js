// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span } from "./mj.js";
import * as util from "./util.js";
const ColorPassword = cc("div", { classes: "ColorPassword" });
export function create_color_pwd(comp) {
    return m("div")
        .addClass("mb-3")
        .append(m("label")
        .addClass("form-label")
        .attr({ for: comp.raw_id })
        .text("Password"), m(comp).addClass("form-textinput form-textinput-fat"), m(ColorPassword).hide(), m("div")
        .addClass("form-text")
        .append("建议使用密码生成器:", util
        .LinkElem("https://www.lastpass.com/features/password-generator", {
        text: "_1_",
        blank: true,
    })
        .addClass("ml-2"), util
        .LinkElem("https://www.random.org/passwords/", {
        text: "_2_",
        blank: true,
    })
        .addClass("ml-2"), "， 易读模式/编辑模式: ", util.LinkElem("#", { text: "toggle" }).attr({ id: 'ColorPwdToggleBtn' }).on("click", (event) => {
        event.preventDefault();
        toggleColorPwd(comp);
    })));
}
function toggleColorPwd(comp) {
    comp.elem().toggle();
    ColorPassword.elem().toggle();
    refreshColorPwd(comp);
    util.focus(comp);
}
function refreshColorPwd(comp) {
    ColorPassword.elem().html("");
    for (let n of util.val(comp, "trim")) {
        if (isNaN(Number(n))) {
            ColorPassword.elem().append(span(n).css({ color: "blue" }));
        }
        else {
            ColorPassword.elem().append(span(n).css({ color: "red" }));
        }
    }
}
