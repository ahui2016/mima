// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const titleArea = m("div")
    .addClass("text-center")
    .append(m("h1").text("Sign in .. mima"));
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
                url: "/sign-out",
                alerts: Alerts,
                buttonID: SignOutBtn.id,
            }, () => {
                Alerts.clear().insert("danger", "已登出");
                SignOutArea.elem().hide();
                SignInForm.elem().show();
                PwdInput.elem().val('');
                util.focus(PwdInput);
            });
        }),
    ],
});
const PwdInput = cc("input");
const SubmitBtn = cc("button");
const SignInForm = cc("form", {
    children: [
        m("label").text("Master Password").attr({ for: PwdInput.raw_id }),
        m("div").append([
            m(PwdInput).attr({ type: "password" }),
            m(SubmitBtn)
                .text("Sign in")
                .on("click", (event) => {
                event.preventDefault();
                const pwd = util.val(PwdInput);
                if (!pwd) {
                    util.focus(PwdInput);
                    return;
                }
                util.ajax({
                    method: "POST",
                    url: "/sign-in",
                    alerts: Alerts,
                    buttonID: SubmitBtn.id,
                    body: { password: pwd },
                }, () => {
                    SignInForm.elem().hide();
                    Alerts.clear().insert('success', '成功登入');
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
            }),
        ]),
    ],
});
$("#root").append(titleArea, m(Loading).hide(), m(SignInForm).hide(), m(Alerts), m(SignOutArea).hide(), m(GotoChangePwd).hide());
init();
function init() {
    checkSignIn();
}
function checkSignIn() {
    util.ajax({ method: "GET", url: "/is-signed-in", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            Alerts.insert('success', '已登入');
            SignOutArea.elem().show();
        }
        else {
            checkDefaultPwd();
        }
    }),
        undefined,
        () => {
            Loading.hide();
        };
}
function checkDefaultPwd() {
    util.ajax({ method: "GET", url: "/is-default-pwd", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            GotoChangePwd.elem().show();
        }
        else {
            SignInForm.elem().show();
            util.focus(PwdInput);
        }
    });
}
