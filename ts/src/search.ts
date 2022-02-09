// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";
import { MimaItem } from "./mima-item.js";

var searchMode: "LabelOnly" | "LabelAndTitle" = "LabelOnly";

const Alerts = util.CreateAlerts(4);
const Loading = util.CreateLoading("center");

const GotoSignOut = cc("a", {
  text: "Sign-out",
  attr: { href: "/public/sign-in.html" },
});

const NaviBar = cc("div", {
  classes: "text-right",
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
    m("div").append(
      span("mode: "),
      m(SearchModeName).text(searchMode),
      util
        .LinkElem("#", { text: "(toggle)" })
        .addClass("ml-1")
        .on("click", (event) => {
          event.preventDefault();
          searchMode =
            searchMode == "LabelOnly" ? "LabelAndTitle" : "LabelOnly";
          SearchModeName.elem().text(searchMode);
        })
    ),
    m(SearchInput),
    m(SearchBtn).on("click", (event) => {
      event.preventDefault();
      const body = { mode: searchMode, pattern: util.val(SearchInput, "trim") };
      Alerts.insert("primary", "正在检索: " + body.pattern);
      util.ajax(
        {
          method: "POST",
          url: "/api/search",
          alerts: Alerts,
          buttonID: SearchBtn.id,
          body: body,
        },
        (resp) => {
          const items = resp as util.Mima[];
          if (items && items.length > 0) {
            Alerts.insert("success", `找到 ${items.length} 条结果`);
            clear_list(MimaList);
            appendToList(MimaList, items.map(MimaItem));
            if (items.length < 5) {
              footerElem.hide();
            } else {
              footerElem.show();
            }
          } else {
            Alerts.insert("info", "找不到。");
          }
        },
        (that, errMsg) => {
          if (that.status == 401) {
            GotoSignIn.elem().show();
          }
          Alerts.insert("danger", errMsg);
        }
      );
    }),
  ],
});

const MimaList = cc("div");

const footerElem = m("div")
  .addClass("Footer")
  .append(
    util
      .LinkElem("https://github.com/ahui2016/mima", { blank: true })
      .addClass("FooterLink")
  );

const TextForCopy = cc("input", { id: "TextForCopy" });

$("#root").append(
  m(NaviBar).addClass("my-3"),
  m(Loading).addClass("my-3"),
  m(SearchForm).hide(),
  m(Alerts),
  m(GotoSignIn).hide(),
  m(MimaList).addClass("mt-3"),
  footerElem.hide(),
  m(TextForCopy).hide()
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
        GotoSignOut.elem().show();
        SearchForm.elem().show();
        util.focus(SearchInput);
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

function clear_list(list: mjComponent): void {
  list.elem().html("");
}
