// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { noConflict } from "jquery";
import { mjElement, mjComponent, m, cc, span, prependToList } from "./mj.js";
import * as util from "./util.js";

const id = util.getUrlParam("id");

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");

const NaviBar = cc("div", {
  children: [util.LinkElem("/", { text: "mima" }), span(" .. Edit an item")],
});

const HistoryList = cc("div");
const HistoryArea = cc("div", {
  children: [m("h3").text("History").addClass("mb-0"), m("hr"), m(HistoryList)],
});

const ID_Input = util.create_input();
const TitleInput = util.create_input();
const LabelInput = util.create_input();
const UsernameInput = util.create_input();
const PasswordInput = util.create_input();
const NotesInput = util.create_textarea();
const FormAlerts = util.CreateAlerts();
const SubmitBtn = cc("button", { text: "Submit" });
const DelBtn = cc("a", {
  attr: { href: "#" },
  text: "Delete",
  classes: "ml-3",
});

const Form = cc("form", {
  children: [
    util.create_item(ID_Input, "ID", ""),
    util.create_item(TitleInput, "Title", "标题（必填）"),
    util.create_item(
      LabelInput,
      "Label",
      "标签，有利于搜索，也可当作分类，不同项目可使用同一个标签。"
    ),
    util.create_item(UsernameInput, "Username", ""),
    util.create_item(PasswordInput, "Password", ""),
    util.create_item(NotesInput, "Notes", ""),
    m(FormAlerts),
    m(SubmitBtn).on("click", (event) => {
      event.preventDefault();
      const title = util.val(TitleInput, "trim");
      if (!title) {
        FormAlerts.insert("danger", "Title(标题)必填");
        util.focus(TitleInput);
        return;
      }
      const body = {
        id: id,
        title: title,
        label: util.val(LabelInput, "trim"),
        username: util.val(UsernameInput, "trim"),
        password: util.val(PasswordInput),
        notes: util.val(NotesInput, "trim"),
      };
      util.ajax(
        {
          method: "POST",
          url: "/api/edit",
          alerts: FormAlerts,
          buttonID: SubmitBtn.id,
          body: body,
        },
        () => {
          Form.elem().hide();
          HistoryArea.elem().hide();
          Alerts.clear().insert("success", `修改成功，可刷新页面查看结果。`);
        }
      );
    }),
    m(DelBtn).on("click", (event) => {
      event.preventDefault();
      util.disable(DelBtn);
      FormAlerts.insert(
        "danger",
        "当 delete 按钮变红时，再点击一次可删除本页内容，不可恢复。"
      );
      setTimeout(() => {
        util.enable(DelBtn);
        DelBtn.elem()
          .css("color", "red")
          .off()
          .on("click", (e) => {
            e.preventDefault();
            util.ajax(
              {
                method: "POST",
                url: "/api/delete-mima",
                alerts: FormAlerts,
                buttonID: DelBtn.id,
                body: { id: id },
              },
              () => {
                FormAlerts.clear().insert(
                  "success",
                  "已彻底删除本页内容，不可恢复"
                );
                SubmitBtn.elem().hide();
                DelBtn.elem().hide();
              }
            );
          });
      }, 2000);
    }),
  ],
});

$("#root").append(
  m(NaviBar).addClass("my-3"),
  m(Loading).addClass("my-3"),
  m(Alerts).addClass("my-3"),
  m(Form).hide(),
  m(HistoryArea).addClass("my-5").hide()
);

init();

function init() {
  if (!id) {
    Loading.hide();
    Alerts.insert("danger", "未指定 id");
    return;
  }
  loadData();
}

function loadData() {
  util.ajax(
    { method: "POST", url: "/api/get-mima", alerts: Alerts, body: { id: id } },
    (resp) => {
      const mwh = resp as util.MimaWithHistory;
      Form.elem().show();
      ID_Input.elem().val(mwh.ID);
      util.disable(ID_Input);
      TitleInput.elem().val(mwh.Title);
      LabelInput.elem().val(mwh.Label);
      UsernameInput.elem().val(mwh.Username);
      PasswordInput.elem().val(mwh.Password);
      NotesInput.elem().val(mwh.Notes);

      if (mwh.History) {
        HistoryArea.elem().show();
        prependToList(HistoryList, mwh.History.map(HistoryItem));
      }
    },
    undefined,
    () => {
      Loading.hide();
    }
  );
}

function HistoryItem(h: util.History): mjComponent {
  const self = cc("div", {
    id: h.ID,
    classes: "HistoryItem",
    children: [
      m("div")
        .addClass("HistoryTitleArea")
        .append(
          span(`(${dayjs.unix(h.CTime).format("YYYY-MM-DD")})`).addClass(
            "text-grey"
          ),
          span(h.Title).addClass("ml-2"),
          util
            .LinkElem("#", { text: "(del)" })
            .attr({ title: "delete" })
            .addClass("delBtn ml-2")
            .on("click", (event) => {
              event.preventDefault();
              const delMsgElem = self.elem().find(".delMsg");
              const delBtnID = self.id + " .delBtn";
              const body = { id: h.ID };
              util.ajax(
                {
                  method: "POST",
                  url: "/api/delete-history",
                  buttonID: delBtnID,
                  body: body,
                },
                () => {
                  $(delBtnID).hide();
                  delMsgElem.show().text("已彻底删除，不可恢复。");
                },
                (_, errMsg) => {
                  const time = dayjs().format("HH:mm:ss");
                  delMsgElem.show().text(`${time} ${errMsg}`);
                }
              );
            })
        ),
      m("div")
        .text("已彻底删除，不可恢复。")
        .addClass("delMsg ml-2 alert-danger")
        .hide(),
      m("div").addClass("UsernamePassword"),
    ],
  });

  self.init = () => {
    const details = self.elem().find(".UsernamePassword");
    if (h.Username) {
      details.append(span("username: ").addClass("text-grey"), h.Username);
    }
    if (h.Password) {
      details.append(span("password: ").addClass("text-grey ml-2"), h.Password);
    }
    if (h.Notes) {
      self
        .elem()
        .append(
          m("div").append(span("Notes: ").addClass("text-grey"), h.Notes)
        );
    }
  };
  return self;
}
