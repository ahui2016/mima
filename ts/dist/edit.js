import { m, cc, span, prependToList } from "./mj.js";
import * as util from "./util.js";
const id = util.getUrlParam("id");
const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");
const NaviBar = cc("div", {
    children: [util.LinkElem("/", { text: "mima" }), span(" .. Edit an item")],
});
const HistoryList = cc("div");
const HistoryArea = cc("div", {
    children: [m("h3").text("History").addClass('mb-0'), m("hr"), m(HistoryList)],
});
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
                HistoryArea.elem().hide();
                Alerts.clear().insert("success", `修改成功，可刷新页面查看结果。`);
            });
        }),
    ],
});
$("#root").append(m(NaviBar).addClass('my-3'), m(Loading).addClass('my-3'), m(Alerts).addClass('my-3'), m(Form).hide(), m(HistoryArea).addClass("my-5").hide());
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
        if (mwh.History) {
            HistoryArea.elem().show();
            prependToList(HistoryList, mwh.History.map(HistoryItem));
        }
    }, undefined, () => {
        Loading.hide();
    });
}
function HistoryItem(h) {
    const self = cc("div", {
        id: h.ID,
        classes: "HistoryItem",
        children: [
            m("div")
                .addClass("HistoryTitleArea")
                .append(span(`(${dayjs.unix(h.CTime).format("YYYY-MM-DD")})`).addClass("text-grey"), span(h.Title).addClass("ml-2")),
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
                .append(m("div").append(span("Notes: ").addClass("text-grey"), h.Notes));
        }
    };
    return self;
}
