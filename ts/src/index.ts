// 采用受 Mithril 启发的基于 jQuery 实现的极简框架 https://github.com/ahui2016/mj.js
import {mjElement, mjComponent, m, cc, span} from './mj.js';
import * as util from './util.js';

const Alerts = util.CreateAlerts();

const titleArea = m('div')
  .addClass('text-center')
  .append(
    m('h1').text("mima")
  );

const DB_Status = cc('div');

$('#root').append(
  titleArea,
  m(Alerts),
  m(DB_Status),
);

init();

function init() {
  util.ajax({method:'GET',url:'/api/is-db-empty',alerts:Alerts}, resp => {
    const isEmpty = resp as boolean;
    DB_Status.elem().text(`${isEmpty}`);
  })
}
