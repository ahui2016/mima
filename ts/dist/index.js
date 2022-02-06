// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const titleArea = m("div").addClass("text-center").append(m("h1").text("mima"));
const NaviBar = cc("div", {
    classes: "text-right mb-3",
    children: [
        util.LinkElem("/public/search.html", { text: "Search" }),
        util.LinkElem("/public/add.html", { text: "Add" }).addClass("ml-2"),
    ],
});
const GotoSignIn = util.CreateGotoSignIn();
const MimaList = cc("div");
$("#root").append(titleArea, m(NaviBar), m(Loading).addClass("my-3"), m(Alerts), m(GotoSignIn).hide(), m(MimaList).addClass("mt-3"));
init();
function init() {
    getAll();
}
function getAll() {
    util.ajax({ method: "GET", url: "/api/all", alerts: Alerts }, (resp) => {
        const all = resp;
        if (all && all.length > 0) {
            appendToList(MimaList, all.map(MimaItem));
        }
        else {
            Alerts.insert("info", "空空如也");
        }
    }, (that, errMsg) => {
        if (that.status == 401) {
            GotoSignIn.elem().show();
        }
        Alerts.insert("danger", errMsg);
    }, () => {
        Loading.hide();
    });
}
function MimaItem(mima) {
    const self = cc("div", {
        id: mima.ID,
        classes: "MimaItem",
        children: [
            m("div")
                .addClass("MimaTitleArea")
                .append(span(`[id: ${mima.ID}]`).addClass("text-grey"), span("").addClass("MimaLabel ml-2").hide(), util
                .LinkElem("/public/edit.html?id=" + mima.ID, {
                text: mima.Title,
                blank: true,
            })
                .addClass("ml-2")),
            m("div").addClass('UsernamePassword'),
        ],
    });
    self.init = () => {
        if (mima.Label) {
            self.elem().find(".MimaLabel").show().text(`[${mima.Label}]`);
        }
        const details = self.elem().find('.UsernamePassword');
        if (mima.Username) {
            details.append(span("username: ").addClass("text-grey"), mima.Username, util.LinkElem("#", { text: "(cp)" }));
        }
        if (mima.Password) {
            details.append(span("password: ").addClass("text-grey ml-2"), '******', util.LinkElem("#", { text: "(cp)" }));
        }
    };
    return self;
}
