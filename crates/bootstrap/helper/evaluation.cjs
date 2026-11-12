'use strict';
// Keep all generated prefix on the original program's first line: source-map line
// ranges emitted by injectra-workspace remain valid after the document guard.
function guardedSource(command, documentKey) {
  const originalSource = [...command.source.matchAll(/^[ \t]*\/\/# sourceURL=(\S+)/gm)].at(-1)?.[1] || 'injectra://unknown';
  const url = 'injectra://request/' + command.request_id + '?revision=' + encodeURIComponent(command.revision) + '&document=' + command.generation + '&source=' + encodeURIComponent(originalSource);
  const source = '(async()=>{if(globalThis[Symbol.for('+JSON.stringify(documentKey)+')]!=='+JSON.stringify(command.generation)+')return {ok:false,certainty:"NotExecuted",error:"Document changed before execution"};try{const value=await ('+command.source+'\n);try{return {ok:true,certainty:"Executed",encoded:JSON.stringify(value===undefined?null:value)}}catch(e){return {ok:false,certainty:"Executed",error:"Result encoding: "+String(e)}}}catch(e){return {ok:false,certainty:"Executed",error:String(e),stack:String(e.stack||"")}}})()\n//# sourceURL='+url;
  return {source,url,context:{request_id:command.request_id,revision:command.revision,page:command.page,generation:command.generation,user_source:originalSource}};
}
module.exports = {guardedSource};
