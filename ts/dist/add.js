// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc, span } from "./mj.js";
import * as util from "./util.js";
import { create_color_pwd } from "./color-password.js";
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading();
const NaviBar = cc("div", {
    children: [
        util.LinkElem("/", { text: "mima" }),
        span(" .. "),
        util.LinkElem("/public/import.html", { text: "Import" }),
        span(" .. Add an item"),
    ],
});
const TitleInput = util.create_input();
const LabelInput = util.create_input();
const UsernameInput = util.create_input();
const PasswordInput = util.create_input();
const NotesInput = util.create_textarea();
const SubmitBtn = cc("button", { text: "Submit" });
const FormAlerts = util.CreateAlerts();
const Form = cc("form", {
    children: [
        util.create_item(TitleInput, "Title", "标题（必填）"),
        util.create_item(LabelInput, "Label", "标签，有利于搜索，也可当作分类，不同项目可使用同一个标签。"),
        util.create_item(UsernameInput, "Username", ""),
        create_color_pwd(PasswordInput),
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
                title: title,
                label: util.val(LabelInput, "trim"),
                username: util.val(UsernameInput, "trim"),
                password: util.val(PasswordInput),
                notes: util.val(NotesInput, "trim"),
            };
            util.ajax({
                method: "POST",
                url: "/api/add",
                alerts: FormAlerts,
                buttonID: SubmitBtn.id,
                body: body,
            }, (resp) => {
                const id = resp.message;
                Form.elem().hide();
                Alerts.clear().insert("success", `已成功添加 (id: ${id})`);
            });
        }),
    ],
});
const GotoSignIn = util.CreateGotoSignIn();
$("#root").append(m(NaviBar).addClass("my-3"), m(Loading).addClass("my-3"), m(Alerts).addClass("my-3"), m(GotoSignIn).hide(), m(Form).hide(), m('div').text('.').addClass('Footer'));
init();
function init() {
    checkSignIn();
}
function checkSignIn() {
    util.ajax({ method: "GET", url: "/auth/is-signed-in", alerts: Alerts }, (resp) => {
        const yes = resp;
        if (yes) {
            Form.elem().show();
            util.focus(TitleInput);
        }
        else {
            GotoSignIn.elem().show();
        }
    }, undefined, () => {
        Loading.hide();
    });
}
