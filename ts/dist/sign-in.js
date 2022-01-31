// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const titleArea = m("div")
    .addClass("text-center")
    .append(m("h1").text("Sign in .. mima"));
const GotoChangePwd = cc("div", {
    children: [
        m("p")
            .addClass("alert-danger")
            .text('默认密码是 "abc", 正式使用前必须先修改密码。'),
        m("div").append("前往修改密码 ➡ ", util.LinkElem("/public/change-pwd.html")),
    ],
});
const Form = cc("div", { text: "The sign-in form" });
$("#root").append(titleArea, m(Loading).hide(), m(Alerts), m(GotoChangePwd).hide(), m(Form).hide());
init();
function init() {
    checkDefaultPwd();
}
function checkDefaultPwd() {
    util.ajax({ method: "GET", url: "/is-default-pwd", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            GotoChangePwd.elem().show();
        }
        else {
            Form.elem().show();
        }
    }),
        undefined,
        () => {
            Loading.hide();
        };
}
