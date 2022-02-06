import { m, cc } from "./mj.js";
import * as util from "./util.js";
const id = util.getUrlParam("id");
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const titleArea = m("div")
    .addClass("text-center")
    .append(m("h1").text("Edit a mima"));
const ID_Input = util.create_input();
const TitleInput = util.create_input();
const LabelInput = util.create_input();
const UsernameInput = util.create_input();
const PasswordInput = util.create_input();
const NotesInput = util.create_textarea();
const FormAlerts = util.CreateAlerts();
const SubmitBtn = cc("button", { text: "Submit" });
const Form = cc("form", {
    children: [
        util.create_item(ID_Input, "ID", ""),
        util.create_item(TitleInput, "Title", "标题（必填）"),
        util.create_item(LabelInput, "Label", "标签，有利于搜索，也可当作分类，不同项目可使用同一个标签。"),
        util.create_item(UsernameInput, "Username", ""),
        util.create_item(PasswordInput, "Password", ""),
        util.create_item(NotesInput, "Notes", ""),
        m(FormAlerts),
        m(SubmitBtn).on("click", (event) => {
            event.preventDefault();
            const title = util.val(TitleInput, "trim");
            if (!title) {
                Alerts.insert("danger", "Title(标题)必填");
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
            util.ajax({
                method: "POST",
                url: "/api/edit",
                alerts: FormAlerts,
                buttonID: SubmitBtn.id,
                body: body,
            }, () => {
                Form.elem().hide();
                Alerts.clear().insert("success", `修改成功，可刷新页面查看结果。`);
            });
        }),
    ],
});
$("#root").append(titleArea, m(Loading), m(Alerts), m(Form).hide());
init();
function init() {
    if (!id) {
        Loading.hide();
        Alerts.insert("danger", "未指定 id");
        return;
    }
    Form.elem().show();
    loadData();
}
function loadData() {
    util.ajax({ method: "POST", url: "/api/get-mima", alerts: Alerts, body: { id: id } }, (resp) => {
        const mwh = resp;
        ID_Input.elem().val(mwh.ID);
        util.disable(ID_Input);
        TitleInput.elem().val(mwh.Title);
        LabelInput.elem().val(mwh.Label);
        UsernameInput.elem().val(mwh.Username);
        PasswordInput.elem().val(mwh.Password);
        NotesInput.elem().val(mwh.Notes);
    }, undefined, () => {
        Loading.hide();
    });
}
