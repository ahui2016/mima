// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span } from "./mj.js";
import * as util from "./util.js";

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading();

const NaviBar = cc("div", {
  classes: "my-5",
  children: [
    util.LinkElem("/", { text: "mima" }),
    span(" .. Change Master Password"),
  ],
});

const DefaultPwdNotes = cc("p", {
  text: "当前密码是 abc, 已自动填写当前密码，请输入新密码。",
  classes: "alert-info",
});

const CurrentPwd = cc("input", {
  attr: { autocomplete: "current-password" },
});
const NewPwd = cc("input", {
  attr: { autocomplete: "new-password" },
});
const SubmitBtn = cc("button", { text: "Change Password" });

const Form = cc("form", {
  children: [
    m("div").append(
      m("label").text("Current Password").attr({ for: CurrentPwd.raw_id }),
      m("br"),
      m(CurrentPwd)
    ),
    m("div").append(
      m("label").text("New Password").attr({ for: NewPwd.raw_id }),
      m("br"),
      m(NewPwd)
    ),
    m(SubmitBtn).on("click", (event) => {
      event.preventDefault();
      const body = {
        oldpwd: util.val(CurrentPwd),
        newpwd: util.val(NewPwd),
      };
      if (!body.oldpwd || !body.newpwd) {
        Alerts.insert("danger", "当前密码与新密码都必填");
        return;
      }
      util.ajax(
        {
          method: "POST",
          url: "/auth/change-pwd",
          alerts: Alerts,
          buttonID: SubmitBtn.id,
          body: body,
        },
        () => {
          Alerts.clear().insert("success", "已成功更改密码。");
          DefaultPwdNotes.elem().hide();
          CurrentPwd.elem().val("");
          NewPwd.elem().val("");
          Form.elem().hide();
        }
      );
    }),
  ],
});

$("#root").append(
  m(NaviBar),
  m(Loading).addClass('my-3'),
  m(DefaultPwdNotes).hide(),
  m(Form),
  m(Alerts)
);

init();

function init() {
  checkDefaultPwd();
}

function checkDefaultPwd() {
  util.ajax(
    { method: "GET", url: "/auth/is-default-pwd", alerts: Alerts },
    (resp) => {
      const yes = resp as boolean;
      if (yes) {
        DefaultPwdNotes.elem().show();
        CurrentPwd.elem().val("abc");
        util.focus(NewPwd);
      } else {
        util.focus(CurrentPwd);
      }
    },
    undefined,
    () => {
      Loading.hide();
    }
  );
}
