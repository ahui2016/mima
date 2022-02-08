// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span } from "./mj.js";
import * as util from "./util.js";

const Alerts = util.CreateAlerts();
const Loading = util.CreateLoading("center");

const NaviBar = cc("div", {
  children: [
    util.LinkElem("/", { text: "mima" }),
    span(" .. Import from JSON"),
    m("p").append(
      "本页专用于导入 mima-web 的数据。(",
      util.LinkElem("https://github.com/ahui2016/mima-web", {
        text: "mima-web",
        blank: true,
      }),
      " 是本程序的前身)"
    ),
  ],
});

const GotoSignIn = util.CreateGotoSignIn();

const FileInput = util.create_input("file");
const UploadBtn = cc("button", { text: "Upload" });

const UploadForm = cc("form", {
  children: [
    m(FileInput).addClass("form-textinput"),
    m(UploadBtn).on("click", (e) => {
      e.preventDefault();
      const body = new FormData();
      const files = (FileInput.elem()[0] as HTMLInputElement).files;
      if (!files || files.length == 0) {
        Alerts.insert("info", "请选择一个文件");
        return;
      }
      body.append("file", files[0]);
      util.ajax(
        {
          method: "POST",
          url: "/api/upload-json",
          alerts: Alerts,
          buttonID: UploadBtn.id,
          // contentType:
          //   "multipart/form-data; boundary=--------abcdefg12345abcde67890abcde",
          body: body,
        },
        () => {
          Alerts.insert("success", "OK");
        }
      );
    }),
  ],
});

$("#root").append(
  m(NaviBar).addClass("my-3"),
  m(Loading).addClass("my-3"),
  m(Alerts),
  m(GotoSignIn).addClass("my-3").hide(),
  m(UploadForm).addClass("my-3").hide()
);

init();

function init() {
  checkSignIn();
}

function checkSignIn() {
  util.ajax(
    { method: "GET", url: "/auth/is-signed-in", alerts: Alerts },
    (resp) => {
      const yes = resp as boolean;
      if (yes) {
        UploadForm.elem().show();
      } else {
        GotoSignIn.elem().show();
      }
    },
    undefined,
    () => {
      Loading.hide();
    }
  );
}
