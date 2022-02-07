// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";
import { MimaItem } from "./mima-item.js";
var searchMode = "LabelOnly";
const Alerts = util.CreateAlerts(4);
const Loading = util.CreateLoading("center");
const titleArea = m("div").append(m("h1").text("mima"));
const GotoSignOut = cc("a", {
    text: "Sign-out",
    attr: { href: "/public/sign-in.html" },
});
const NaviBar = cc("div", {
    classes: "text-right mb-5",
    children: [
        util.LinkElem("/public/index.html", { text: "Index" }),
        util.LinkElem("/public/add.html", { text: "Add" }).addClass("ml-2"),
        m(GotoSignOut).addClass("ml-2").hide(),
    ],
});
const GotoSignIn = util.CreateGotoSignIn();
const SearchModeName = cc("span");
const SearchInput = cc("input");
const SearchBtn = cc("button", { text: "search" });
const SearchForm = cc("form", {
    children: [
        m("div").append(span("mode: "), m(SearchModeName).text(searchMode), util
            .LinkElem("#", { text: "(toggle)" })
            .addClass("ml-1")
            .on("click", (event) => {
            event.preventDefault();
            searchMode =
                searchMode == "LabelOnly" ? "LabelAndTitle" : "LabelOnly";
            SearchModeName.elem().text(searchMode);
        })),
        m(SearchInput),
        m(SearchBtn).on("click", (event) => {
            event.preventDefault();
            const body = { mode: searchMode, pattern: util.val(SearchInput, "trim") };
            Alerts.insert("primary", "正在检索: " + body.pattern);
            util.ajax({
                method: "POST",
                url: "/api/search",
                alerts: Alerts,
                buttonID: SearchBtn.id,
                body: body,
            }, (resp) => {
                const items = resp;
                if (items && items.length > 0) {
                    Alerts.insert("success", `找到 ${items.length} 条结果`);
                    clear_list(MimaList);
                    appendToList(MimaList, items.map(MimaItem));
                }
                else {
                    Alerts.insert("info", "找不到。");
                }
            }, (that, errMsg) => {
                if (that.status == 401) {
                    GotoSignIn.elem().show();
                }
                Alerts.insert("danger", errMsg);
            });
        }),
    ],
});
const MimaList = cc("div");
const TextForCopy = cc('input', { id: 'TextForCopy' });
$("#root").append(titleArea, m(NaviBar), m(Loading).addClass("my-3"), m(SearchForm).hide(), m(Alerts), m(GotoSignIn).hide(), m(MimaList).addClass("mt-3"), m(TextForCopy).hide());
init();
function init() {
    checkSignIn();
}
function checkSignIn() {
    util.ajax({ method: "GET", url: "/auth/is-signed-in", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            GotoSignOut.elem().show();
            SearchForm.elem().show();
            util.focus(SearchInput);
        }
        else {
            GotoSignIn.elem().show();
        }
    }, undefined, () => {
        Loading.hide();
    });
}
function clear_list(list) {
    list.elem().html("");
}
