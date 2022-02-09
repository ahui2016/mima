// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span } from "./mj.js";
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
const PwdAlerts = util.CreateAlerts();
const Form = cc("form", {
    children: [
        m("div").append(m("label").text("Current Password").attr({ for: CurrentPwd.raw_id }), m("br"), m(CurrentPwd)),
        m("div").append(m("label").text("New Password").attr({ for: NewPwd.raw_id }), m("br"), m(NewPwd)),
        m(PwdAlerts),
        m(SubmitBtn).on("click", (event) => {
            event.preventDefault();
            const body = {
                oldpwd: util.val(CurrentPwd),
                newpwd: util.val(NewPwd),
            };
            if (!body.oldpwd || !body.newpwd) {
                PwdAlerts.insert("danger", "当前密码与新密码都必填");
                return;
            }
            util.ajax({
                method: "POST",
                url: "/auth/change-pwd",
                alerts: PwdAlerts,
                buttonID: SubmitBtn.id,
                body: body,
            }, () => {
                Alerts.clear().insert("success", "已成功更改密码。");
                DefaultPwdNotes.elem().hide();
                CurrentPwd.elem().val("");
                NewPwd.elem().val("");
                Form.elem().hide();
            });
        }),
    ],
});
const AboutPIN = cc("p", {
    text: "PIN 码是指一个更简单的密码，通过受信任列表中的 IP 访问时可使用 PIN 码登入。你可在此更改 PIN 码 (默认: 1234)。",
});
const CurrentPIN = cc("input", {
    attr: { autocomplete: "current-password" },
});
const NewPIN = cc("input", {
    attr: { autocomplete: "new-password" },
});
const ChangePinBtn = cc("button", { text: "Change PIN" });
const PinAlerts = util.CreateAlerts();
const PinForm = cc("form", {
    children: [
        m("div").append(m("label").text("Current PIN").attr({ for: CurrentPIN.raw_id }), m("br"), m(CurrentPIN)),
        m("div").append(m("label").text("New PIN").attr({ for: NewPIN.raw_id }), m("br"), m(NewPIN)),
        m(PinAlerts),
        m(ChangePinBtn).on("click", (event) => {
            event.preventDefault();
            const body = {
                oldpwd: util.val(CurrentPIN),
                newpwd: util.val(NewPIN),
            };
            if (!body.oldpwd || !body.newpwd) {
                PinAlerts.insert("danger", "当前PIN码与新PIN码都必须填写");
                return;
            }
            util.ajax({
                method: "POST",
                url: "/auth/change-pin",
                alerts: PinAlerts,
                buttonID: SubmitBtn.id,
                body: body,
            }, () => {
                Alerts.clear().insert("success", "已成功更改PIN码。");
                AboutPIN.elem().hide();
                CurrentPIN.elem().val("");
                NewPIN.elem().val("");
                PinForm.elem().hide();
            });
        }),
    ],
});
$("#root").append(m(NaviBar), m(Loading).addClass('my-3'), m(DefaultPwdNotes).hide(), m(Form), m(AboutPIN).addClass('mt-5'), m(PinForm), m(Alerts));
init();
function init() {
    checkDefaultPwd();
}
function checkDefaultPwd() {
    util.ajax({ method: "GET", url: "/auth/is-default-pwd", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            DefaultPwdNotes.elem().show();
            CurrentPwd.elem().val("abc");
            util.focus(NewPwd);
        }
        else {
            util.focus(CurrentPwd);
        }
    }, undefined, () => {
        Loading.hide();
    });
}
