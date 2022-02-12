// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const footerElem = util.CreateFooter();
const NaviBar = cc("div", {
    classes: "my-5",
    children: [
        util.LinkElem("/", { text: "mima" }),
        span(" .. "),
        util.LinkElem("/public/change-pwd.html", { text: "change password" }),
        span(" .. Sign-in"),
    ],
});
const GotoChangePwd = cc("div", {
    children: [
        m("p")
            .addClass("alert-danger")
            .text('默认密码是 "abc", 正式使用前必须先修改密码。'),
        m("div").append("前往修改密码 ➡ ", util.LinkElem("/public/change-pwd.html")),
    ],
});
const SignOutBtn = cc("button");
const SignOutArea = cc("div", {
    children: [
        m(SignOutBtn)
            .text("Sign out")
            .on("click", (event) => {
            event.preventDefault();
            util.ajax({
                method: "GET",
                url: "/auth/sign-out",
                alerts: Alerts,
                buttonID: SignOutBtn.id,
            }, () => {
                Alerts.clear().insert("info", "已登出");
                SignOutArea.elem().hide();
                SignInForm.elem().show();
                util.focus(PwdInput);
            });
        }),
    ],
});
// https://www.chromium.org/developers/design-documents/form-styles-that-chromium-understands/
const UsernameInput = cc("input", { attr: { autocomplete: "username" } });
const PwdInput = cc("input", { attr: { autocomplete: "current-password" } });
const SubmitBtn = cc("button", { text: "Sign in" });
const SignInForm = cc("form", {
    children: [
        m("label").text("Master Password").attr({ for: PwdInput.raw_id }),
        m("div").append(m(UsernameInput).hide(), m(PwdInput).attr({ type: "password" }), m(SubmitBtn)
            .addClass("ml-1")
            .on("click", (event) => {
            event.preventDefault();
            const pwd = util.val(PwdInput);
            if (!pwd) {
                util.focus(PwdInput);
                return;
            }
            util.ajax({
                method: "POST",
                url: "/auth/sign-in",
                alerts: Alerts,
                buttonID: SubmitBtn.id,
                body: { password: pwd },
            }, () => {
                PwdInput.elem().val("");
                SignInForm.elem().hide();
                Alerts.clear().insert("success", "成功登入");
                setMyIP();
                SignOutArea.elem().show();
            }, (that, errMsg) => {
                if (that.status == 401) {
                    Alerts.insert("danger", "密码错误");
                }
                else {
                    Alerts.insert("danger", errMsg);
                }
            }, () => {
                util.focus(PwdInput);
            });
        })),
    ],
});
function myIPElem() {
    return span("").addClass("MyIP");
}
function gotoTrusted(text = "白名单") {
    return util.LinkElem("/public/change-pwd.html", {
        text: text,
        blank: true,
    });
}
const TrustedIP_Area = cc("div", {
    children: [
        span("当前 IP 已受信任: "),
        myIPElem(),
        gotoTrusted("(受信任IP清单)").addClass("ml-2"),
    ],
});
const PinInput = util.create_input('password');
const AddIP_Btn = cc("button", { text: "Trust" });
const IP_Alerts = util.CreateAlerts();
const IP_Area = cc("form", {
    children: [
        m("div").append("当前 IP 如下所示，输入 PIN 码并点击 Trust 按钮可添加到", gotoTrusted(), "。通过白名单中的 IP 访问时可使用 PIN 码登入。"),
        m("div")
            .append(myIPElem(), m(PinInput)
            .attr({ placeholder: "PIN" })
            .addClass("ml-3")
            .css({ width: "100px" }), m(AddIP_Btn)
            .addClass("ml-1")
            .on("click", (e) => {
            e.preventDefault();
            const pin = util.val(PinInput);
            if (!pin) {
                IP_Alerts.insert("info", "请输入PIN码");
                PinInput.elem().trigger('focus');
                return;
            }
            util.ajax({
                method: "POST",
                url: "/api/add-trusted-ip",
                alerts: IP_Alerts,
                buttonID: AddIP_Btn.id,
                body: { password: pin },
            }, () => {
                IP_Alerts.clear().insert("success", "添加信任 IP 成功");
                IP_Area.elem().hide();
            });
        }))
            .addClass("mt-2"),
    ],
});
$("#root").append(m(NaviBar), m(Loading).addClass("my-3"), m(SignInForm).hide(), m(Alerts), m(TrustedIP_Area).addClass("my-5").hide(), m(SignOutArea).addClass("my-5").hide(), m(IP_Area).addClass("mt-5").hide(), m(IP_Alerts), m(GotoChangePwd).hide(), footerElem.hide());
init();
function init() {
    checkSignIn();
}
function checkSignIn() {
    util.ajax({ method: "GET", url: "/auth/is-signed-in", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            Alerts.insert("info", "已登入");
            SignOutArea.elem().show();
            Loading.hide();
            setMyIP();
        }
        else {
            checkDefaultPwd();
        }
    });
}
function checkDefaultPwd() {
    util.ajax({ method: "GET", url: "/auth/is-default-pwd", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            GotoChangePwd.elem().show();
        }
        else {
            SignInForm.elem().show();
            util.focus(PwdInput);
        }
    }, undefined, () => {
        Loading.hide();
    });
}
function setMyIP() {
    util.ajax({ method: "GET", url: "/api/get-my-ip", alerts: Alerts }, (resp) => {
        if (resp.Trusted) {
            TrustedIP_Area.elem().show();
        }
        else {
            IP_Area.elem().show();
            util.focus(PinInput);
        }
        $(".MyIP").text(resp.IP);
        footerElem.show();
    });
}
