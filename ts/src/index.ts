// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span } from "./mj.js";
import * as util from "./util.js";

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");

const titleArea = m("div").addClass("text-center").append(m("h1").text("mima"));

const GotoSignIn = cc("div", {
  children: [
    m("p").addClass("alert-danger").text("请先登入。"),
    m("div").append("前往登入页面 ➡ ", util.LinkElem("/public/sign-in.html")),
  ],
});

$("#root").append(
  titleArea,
  m(Loading).hide(),
  m(Alerts),
  m(GotoSignIn).hide()
);

init();

function init() {
  checkSignIn();
}

function checkSignIn() {
  util.ajax({ method: "GET", url: "/is-signed-in", alerts: Alerts }, (resp) => {
    const yes = resp as boolean;
    if (!yes) {
      GotoSignIn.elem().show();
    }
  }),
    undefined,
    () => {
      Loading.hide();
    };
}
