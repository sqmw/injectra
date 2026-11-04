import {readFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {requireTool} from '../../scripts/test-env.mjs';
const {JSDOM}=requireTool('jsdom');
const programs=JSON.parse(readFileSync(process.argv[2]));
const dom=new JSDOM('<!doctype html><html><head></head><body></body></html>',{runScripts:'outside-only'});
try{
 assert.equal((await dom.window.eval(programs.apply)).status,'applied');
 const panel=dom.window.document.getElementById('injectra-douyin-proof');
 assert.equal(panel.dataset.executed,'1');
 panel.querySelector('button').click();
 assert.equal(panel.querySelector('button').dataset.count,'1');
 assert.equal((await dom.window.eval(programs.apply)).status,'updated');
 assert.equal(dom.window.document.querySelectorAll('#injectra-douyin-proof').length,1);
 assert.equal((await dom.window.eval(programs.remove)).status,'removed');
 assert.equal(dom.window.document.querySelector('#injectra-douyin-proof'),null);
 console.log('Shared generated payload smoke OK (offline DOM; not live compatibility)');
}finally{dom.window.close();}
