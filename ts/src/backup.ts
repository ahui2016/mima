// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span } from "./mj.js";
import * as util from "./util.js";

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");

const NaviBar = cc("div", {
  children: [
    util.LinkElem("/", { text: "mima" }),
    span(" .. Download backup file"),
  ],
});

const GotoSignIn = util.CreateGotoSignIn();

const DownloadArea = cc("div", {
  children: [
    m("p").text(
      "点击下面的链接（或右键点击后选择“另存为”）可下载数据库文件（已加密的数据）："
    ),
    m("p").append(
      util
        .LinkElem("/api/download-backup", { text: "sqlite database file" })
        .attr({ downlaod: "db-mima.sqlite" })
    ),
  ],
});

$("#root").append(
  m(NaviBar).addClass("my-3"),
  m(Loading).addClass("my-3"),
  m(Alerts),
  m(DownloadArea).addClass('my-5').hide(),
  m(GotoSignIn).addClass("my-5").hide(),
);

init();

function init() {
  checkSignIn();
}

function checkSignIn() {
  util.ajax(
    { method: "GET", url: "/auth/is-signed-in", alerts: Alerts },
    (resp) => {
      const yes = resp as boolean;
      if (yes) {
        DownloadArea.elem().show();
      } else {
        GotoSignIn.elem().show();
      }
    },
    undefined,
    () => {
      Loading.hide();
    }
  );
}
