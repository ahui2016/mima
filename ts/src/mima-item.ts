// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import { mjElement, mjComponent, m, cc, span, appendToList } from "./mj.js";
import * as util from "./util.js";

export function MimaItem(mima: util.Mima): mjComponent {
  const ItemAlerts = util.CreateAlerts();
  const self = cc("div", {
    id: mima.ID,
    classes: "MimaItem",
    children: [
      m("div")
        .addClass("MimaTitleArea")
        .append(
          span(`[id: ${mima.ID}]`).addClass("text-grey"),
          span("").addClass("MimaLabel ml-2").hide(),
          util
            .LinkElem("/public/edit.html?id=" + mima.ID, {
              text: mima.Title,
              blank: true,
            })
            .addClass("font-bold ml-2")
        ),
      m("div").addClass("UsernamePassword"),
      m(ItemAlerts),
    ],
  });
  self.init = () => {
    if (mima.Label) {
      self.elem().find(".MimaLabel").show().text(`[${mima.Label}]`);
    }
    const details = self.elem().find(".UsernamePassword");
    if (mima.Username) {
      details.append(
        span("username: ").addClass("text-grey"),
        mima.Username,
        util.LinkElem("#", { text: "(cp)" }).on("click", (e) => {
          e.preventDefault();
          copyToClipboard(mima.Username);
          ItemAlerts.insert("success", "复制用户名 成功");
        })
      );
    }
    if (mima.Password) {
      details.append(
        span("password: ").addClass("text-grey ml-2"),
        "******",
        util
          .LinkElem("#", { text: "(cp)" })
          .addClass("GetPwdBtn")
          .on("click", (e) => {
            e.preventDefault();
            const btnID = self.id + " .GetPwdBtn";
            util.ajax(
              {
                method: "POST",
                url: "/api/get-pwd",
                alerts: ItemAlerts,
                buttonID: btnID,
                body: { id: mima.ID },
              },
              (resp) => {
                const pwd = (resp as util.Text).message;
                copyToClipboard(pwd);
                ItemAlerts.insert("success", "复制密码 成功");
              }
            );
          })
      );
    }
  };
  return self;
}

function copyToClipboard(s: string): void {
  const textElem = $("#TextForCopy");
  textElem.show();
  textElem.val(s).trigger("select");
  document.execCommand("copy"); // execCommand 准备退役了，但仍没有替代方案，因此继续用。
  textElem.val("");
  textElem.hide();
}
