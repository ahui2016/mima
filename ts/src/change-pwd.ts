// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import {mjElement, mjComponent, m, cc, span} from './mj.js';
import * as util from './util.js';

const Alerts = util.CreateAlerts();

const titleArea = m('div')
  .addClass('text-center')
  .append(
    m('h2').text("Change Master Password .. mima")
  );

$('#root').append(
  titleArea,
  m(Alerts),
);

init();

function init() {
}
