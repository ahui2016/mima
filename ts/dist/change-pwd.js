// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading();
const footerElem = util.CreateFooter();
const NaviBar = cc("div", {
    classes: "my-5",
    children: [util.LinkElem("/", { text: "mima" }), span(" .. Change Password")],
});
const AboutDefaultPwd = cc("p", {
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
        m("h4").text("Master Password (主密码)").addClass("mb-0"),
        m("hr"),
        m(AboutDefaultPwd).hide(),
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
                AboutDefaultPwd.elem().hide();
                CurrentPwd.elem().val("");
                NewPwd.elem().val("");
                Form.elem().hide();
            });
        }),
    ],
});
const NewPIN = cc("input");
const SetPinBtn = cc("button", { text: "Set PIN" });
const PinAlerts = util.CreateAlerts();
const PinForm = cc("form", {
    children: [
        m("h4").text("PIN 码").addClass("mb-0"),
        m("hr"),
        m("p").append("PIN 码是指一个更简单的密码，通过受信任列表中的 IP 访问时可使用 PIN 码登入。", "你可在此设置 PIN 码（如未设置则默认为'1234'）。"),
        m("div").append(m("label").text("New PIN").attr({ for: NewPIN.raw_id }), m("br"), m(NewPIN)),
        m(PinAlerts),
        m(SetPinBtn).on("click", (event) => {
            event.preventDefault();
            const body = {
                oldpwd: "******",
                newpwd: util.val(NewPIN),
            };
            if (!body.newpwd) {
                NewPIN.elem().trigger("focus");
                return;
            }
            util.ajax({
                method: "POST",
                url: "/api/change-pin",
                alerts: PinAlerts,
                buttonID: SetPinBtn.id,
                body: body,
            }, () => {
                PinAlerts.clear().insert("success", "已成功更改PIN码。");
                NewPIN.elem().val("");
            });
        }),
    ],
});
const IP_List = cc("ul");
const ClearIP_Alerts = util.CreateAlerts();
const ClearIP_Btn = cc("button", { text: "Clear" });
const IP_ListArea = cc("div", {
    children: [
        m("h4").text("Trusted IPs (受信任IP清单)").addClass("mb-0"),
        m("hr"),
        m("div").text("受信任的 IP 可使用 PIN 码登入，点击 Clear 按钮可清空列表。"),
        m(IP_List),
        m(ClearIP_Alerts),
        m(ClearIP_Btn).on("click", (e) => {
            e.preventDefault();
            util.ajax({
                method: "POST",
                url: "/api/clear-trusted-ips",
                alerts: ClearIP_Alerts,
                buttonID: ClearIP_Btn.id,
            }, () => {
                IP_List.elem().html("");
                ClearIP_Alerts.insert("success", "已清空");
            });
        }),
    ],
});
$("#root").append(m(NaviBar), m(Loading).addClass("my-3"), m(Alerts), m(Form).addClass("my-5"), m(PinForm).addClass("my-5").hide(), m(IP_ListArea).addClass("my-5"), footerElem);
init();
function init() {
    checkDefaultPwd();
    checkSignIn();
}
function checkDefaultPwd() {
    util.ajax({ method: "GET", url: "/auth/is-default-pwd", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            AboutDefaultPwd.elem().show();
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
function checkSignIn() {
    util.ajax({ method: "GET", url: "/auth/is-signed-in", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            PinForm.elem().show();
            init_ip_list();
        }
        else {
            ClearIP_Alerts.insert("info", "Required Sign-in (登入后才能查看此项)");
        }
    });
}
function init_ip_list() {
    util.ajax({ method: "GET", url: "/api/get-trusted-ips", alerts: Alerts }, (resp) => {
        const ips = resp;
        if (ips && ips.length > 0) {
            ips.forEach((ip) => {
                IP_List.elem().append(m("li").text(ip));
            });
        }
        else {
            ClearIP_Alerts.insert("info", "未添加信任IP");
        }
    });
}
