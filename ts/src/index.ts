// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";
import { MimaItem } from "./mima-item.js";

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const footerElem = util.CreateFooter();

const titleArea = m("div").addClass("text-center").append(m("h1").text("mima"));

const NaviBar = cc("div", {
  classes: "text-right",
  children: [
    util.LinkElem("/public/search.html", { text: "Search" }),
    util.LinkElem("/public/add.html", { text: "Add" }).addClass("ml-2"),
    util.LinkElem("/public/backup.html", { text: "Backup" }).addClass("ml-2"),
  ],
});

const GotoSignIn = util.CreateGotoSignIn();

const MimaList = cc("div");

const TextForCopy = cc("input", { id: "TextForCopy" });

$("#root").append(
  titleArea,
  m(NaviBar),
  m(Loading).addClass("my-3"),
  m(Alerts),
  m(GotoSignIn).addClass("my-3").hide(),
  m(MimaList).addClass("mt-3"),
  footerElem.hide(),
  m(TextForCopy).hide(),
);

init();

function init() {
  getAll();
}

function getAll() {
  util.ajax(
    { method: "GET", url: "/api/all", alerts: Alerts },
    (resp) => {
      const all = resp as util.Mima[];
      if (all && all.length > 0) {
        appendToList(MimaList, all.map(MimaItem));
        if (all.length >= 5) {
          footerElem.show();
        }
      } else {
        Alerts.insert("info", "空空如也");
      }
    },
    (that, errMsg) => {
      if (that.status == 401) {
        GotoSignIn.elem().show();
      }
      Alerts.insert("danger", errMsg);
    },
    () => {
      Loading.hide();
    }
  );
}
