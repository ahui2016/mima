// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";

var searchMode: "LabelOnly" | "LabelAndTitle" = "LabelOnly";

const Alerts = util.CreateAlerts(4);
const Loading = util.CreateLoading("center");

const titleArea = m("div")
  .addClass("text-center")
  .append(m("h1").text("Search mima"));

const GotoSignIn = cc("div", {
  children: [
    m("p").addClass("alert-danger").text("请先登入。"),
    m("div").append("前往登入页面 ➡ ", util.LinkElem("/public/sign-in.html")),
  ],
});

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
          } else {
            Alerts.insert('info', '找不到。');
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

$("#root").append(
  titleArea,
  m(Loading),
  m(SearchForm).hide(),
  m(Alerts),
  m(GotoSignIn).hide(),
  m(MimaList).addClass("mt-3")
);

init();

function init() {
  checkSignIn();
}

function MimaItem(mima: util.Mima): mjComponent {
  const self = cc("div", {
    id: mima.ID,
    classes: "MimaItem",
    children: [
      m("div")
        .addClass("MimaTitleArea")
        .append(
          span(`[id: ${mima.ID}]`).addClass("text-grey"),
          span("").addClass("MimaLabel ml-2").hide(),
          util
            .LinkElem("/public/edit.html?id=" + mima.ID, {
              text: mima.Title,
              blank: true,
            })
            .addClass("ml-2")
        ),
      m("div").append(
        span("username: ").addClass("text-grey"),
        mima.Username,
        util.LinkElem("#", { text: "(cp)" }),
        span("password: ").addClass("text-grey ml-2"),
        util.LinkElem("#", { text: "(cp)" })
      ),
    ],
  });
  self.init = () => {
    if (mima.Label) {
      self.elem().find(".MimaLabel").show().text(`[${mima.Label}]`);
    }
  };
  return self;
}

function checkSignIn() {
  util.ajax(
    { method: "GET", url: "/auth/is-signed-in", alerts: Alerts },
    (resp) => {
      const yes = resp as boolean;
      if (yes) {
        SearchForm.elem().show();
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
  list.elem().html('');
}
