// Thunderbird-specific test host adapter. Does not patch Gecko's TabManager.
import { pageWorldScript } from "./page-world.mjs";
export class ThunderbirdContentHost {
  constructor(marionette, sessionId) { this.client = marionette; this.sessionId = sessionId; }
  async list() {
    const pages = await this.client.evaluate("document.getElementById('tabmail').tabInfo.filter(t=>" +
      "t.mode.name==='contentTab'&&t.browser?.browsingContext?.currentWindowGlobal).map(t=>({" +
      "context:String(t.browser.browsingContext.id)," +
      "generation:String(t.browser.browsingContext.currentWindowGlobal.innerWindowId)," +
      "url:t.browser.currentURI.spec,title:t.browser.contentTitle,loading:t.browser.webProgress.isLoadingDocument}))");
    return pages.map(page => ({ ...page, sessionId: this.sessionId }));
  }
  select(target) {
    if (target.sessionId !== this.sessionId) throw new Error("Target belongs to another test session");
    return "const target=" + JSON.stringify(target) + ";" +
      "const tabmail=document.getElementById('tabmail');" +
      "const tab=tabmail.tabInfo.find(t=>t.mode.name==='contentTab'&&" +
      "String(t.browser?.browsingContext?.id)===target.context);" +
      "if(!tab)throw Error('Thunderbird target not found');" +
      "const browser=tab.browser;const windowGlobal=browser.browsingContext.currentWindowGlobal;" +
      "if(!windowGlobal||String(windowGlobal.innerWindowId)!==target.generation||browser.currentURI.spec!==target.url)" +
      "throw Error('Thunderbird target generation changed');";
  }
  async evaluate(target, expression) {
    const script = pageWorldScript(expression);
    const value = await this.client.evaluate("(async()=>{" + this.select(target) +
      "return await windowGlobal.getActor('MarionetteCommands').executeScript(" +
      JSON.stringify(script) + ",[],{sandboxName:\"system\",newSandbox:true,timeout:6000,async:false});})()");
    // The actor returns the page string; external Marionette JSON framing is already decoded.
    if (typeof value !== "string") throw new Error("Expected serialized content result from Gecko actor");
    return JSON.parse(value);
  }
  async screenshot(target) {
    await this.client.evaluate("(()=>{" + this.select(target) + "tabmail.switchToTab(tab);return true;})()");
    const image = await this.client.command("WebDriver:TakeScreenshot", { full: false });
    return image.value;
  }
  async navigate(target, url) {
    await this.client.evaluate("(()=>{" + this.select(target) +
      "const {MailE10SUtils}=ChromeUtils.importESModule('resource:///modules/MailE10SUtils.sys.mjs');" +
      "MailE10SUtils.loadURI(browser," + JSON.stringify(url) + ");return true;})()");
  }
  async close(target) {
    await this.client.evaluate("(()=>{" + this.select(target) + "tabmail.closeTab(tab);return true;})()");
  }
}
