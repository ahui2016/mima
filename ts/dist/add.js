// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { m, cc } from "./mj.js";
import * as util from "./util.js";
const Alerts = util.CreateAlerts();
const titleArea = m("div")
    .addClass("text-center")
    .append(m("h1").text("Add a mima"));
const TitleInput = util.create_input();
const LabelInput = util.create_input();
const UsernameInput = util.create_input();
const PasswordInput = util.create_input();
const NotesInput = util.create_textarea();
const SubmitBtn = cc("button", { text: "Submit" });
const Form = cc("form", {
    children: [
        util.create_item(TitleInput, "Title", "标题（必填）"),
        util.create_item(LabelInput, "Label", "标签，有利于搜索，也可当作分类，不同项目可使用同一个标签。"),
        util.create_item(UsernameInput, "Username", ""),
        util.create_item(PasswordInput, "Password", ""),
        util.create_item(NotesInput, "Notes", ""),
        m(SubmitBtn).on("click", (event) => {
            event.preventDefault();
            const title = util.val(TitleInput, "trim");
            if (!title) {
                Alerts.insert("danger", "Title(标题)必填");
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
                alerts: Alerts,
                buttonID: SubmitBtn.id,
                body: body,
            }, (resp) => {
                const id = resp.message;
                Form.elem().hide();
                Alerts.clear().insert("success", `已成功添加 (id:${id})`);
            });
        }),
    ],
});
$("#root").append(titleArea, m(Form), m(Alerts));
init();
function init() {
    util.focus(TitleInput);
}
