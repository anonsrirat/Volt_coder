// Lightweight i18n — no dependency, just a typed dictionary + hook.

import { useSettingsStore } from "./store";

export type Lang = "en" | "th";

type Dict = Record<string, string>;

const en: Dict = {
  // app chrome
  "app.tagline": "coding agent",
  "app.starting": "Starting Volt…",
  // sidebar
  "sidebar.newChat": "New chat",
  "sidebar.openFolder": "Open folder",
  "sidebar.history": "History",
  "sidebar.noConversations": "No conversations yet",
  // explorer
  "explorer.title": "Explorer",
  "explorer.empty": "No folder open. Use Open folder to begin.",
  // chat
  "chat.placeholder": "Ask Volt to build, explain or refactor…",
  "chat.placeholderNoProvider": "Add a provider in Settings to start chatting",
  "chat.send": "Send",
  "chat.stop": "Stop",
  "chat.note": "Volt can read, edit and run code in your project. Actions that change files require your approval.",
  "chat.welcomeSub": "Your fast, project-aware AI coding agent.",
  "chat.noProvider": "No provider configured yet — open Settings to add your API key.",
  "chat.thinking": "Thinking level",
  "chat.context": "Context",
  "chat.sendKey": "to send",
  "chat.newlineKey": "for a new line",
  "thinking.auto": "Auto",
  "thinking.off": "Off",
  "thinking.low": "Low",
  "thinking.medium": "Medium",
  "thinking.high": "High",
  "thinking.auto.hint": "Let the model decide how much to reason.",
  "thinking.off.hint": "Fastest — no extended reasoning.",
  "thinking.low.hint": "A little reasoning for simple tasks.",
  "thinking.medium.hint": "Balanced reasoning for most tasks.",
  "thinking.high.hint": "Deep, thorough reasoning for hard tasks.",
  // approval
  "approval.required": "Approval required",
  "approval.wantModify": "Volt wants to modify files in your project.",
  "approval.wantRun": "Volt wants to run a command in your project.",
  "approval.approve": "Approve",
  "approval.deny": "Deny",
  // settings
  "settings.title": "Settings",
  "settings.providers": "Providers",
  "settings.appearance": "Appearance",
  "settings.language": "Language",
  "settings.general": "General",
  "settings.mcp": "MCP Servers",
  "settings.plugins": "Skills & Plugins",
  "settings.about": "About",
  "settings.addProvider": "Add provider",
  "settings.noProviders": "No providers yet. Add one to start using Volt.",
  "settings.active": "active",
  "settings.use": "Use",
  "settings.editProvider": "Edit provider",
  "settings.type": "Type",
  "settings.name": "Display name",
  "settings.baseUrl": "Base URL",
  "settings.model": "Model",
  "settings.apiKey": "API key",
  "settings.keyHint": "Stored securely in your OS keyring — never written to disk in plaintext.",
  "settings.useBearer": "Send Authorization: Bearer header",
  "settings.test": "Test connection",
  "settings.save": "Save provider",
  "settings.back": "Back",
  // theme
  "theme.dark": "Dark",
  "theme.light": "Light",
  "theme.system": "System",
  "theme.accent": "Accent colour",
  // general
  "general.approval": "Approval policy",
  "general.approvalWrite": "Ask before writing files",
  "general.approvalRun": "Ask before running commands",
  "general.approvalRead": "Ask before reading files",
  "general.compaction": "Context compaction",
  "general.compactionEnable": "Enable auto-compaction",
  "general.compactionThreshold": "Compact at",
  "general.maxIterations": "Max agent iterations",
  "general.autoUpdate": "Check for updates on launch",
  // mcp
  "mcp.add": "Add server",
  "mcp.empty": "No MCP servers configured. Connect tools like filesystem, git or fetch.",
  "mcp.connect": "Connect",
  "mcp.disconnect": "Disconnect",
  "mcp.connected": "Connected",
  "mcp.transport": "Transport",
  "mcp.command": "Command",
  "mcp.url": "URL",
  // plugins
  "plugins.add": "Add skill",
  "plugins.empty": "No skills yet. Add custom prompt skills to extend Volt.",
  "plugins.prompt": "Prompt instructions",
  "plugins.command": "Slash command",
  // about
  "about.version": "Version",
  "about.repo": "GitHub repository",
  "about.checkUpdates": "Check for updates",
  "about.upToDate": "You're on the latest version.",
  "about.updateAvailable": "Update available",
  "about.download": "Download & install",
  "about.downloading": "Downloading…",
  // update banner
  "update.newVersion": "Volt {version} is available",
  "update.view": "View",
  "update.later": "Later",
  // canvas
  "canvas.title": "Plan",
  "canvas.empty": "No active plan. Ask Volt to plan a multi-step task.",
  "canvas.pending": "Pending",
  "canvas.inProgress": "In progress",
  "canvas.completed": "Completed",
  "canvas.compacted": "Context compacted",
};

const th: Dict = {
  "app.tagline": "เอไอเขียนโค้ด",
  "app.starting": "กำลังเปิด Volt…",
  "sidebar.newChat": "แชทใหม่",
  "sidebar.openFolder": "เปิดโฟลเดอร์",
  "sidebar.history": "ประวัติ",
  "sidebar.noConversations": "ยังไม่มีการสนทนา",
  "explorer.title": "ไฟล์โปรเจกต์",
  "explorer.empty": "ยังไม่ได้เปิดโฟลเดอร์ กด เปิดโฟลเดอร์ เพื่อเริ่ม",
  "chat.placeholder": "บอก Volt ให้สร้าง อธิบาย หรือปรับโครงสร้างโค้ด…",
  "chat.placeholderNoProvider": "เพิ่มผู้ให้บริการในตั้งค่าก่อนเริ่มแชท",
  "chat.send": "ส่ง",
  "chat.stop": "หยุด",
  "chat.note": "Volt อ่าน แก้ไข และรันโค้ดในโปรเจกต์ได้ การเปลี่ยนแปลงไฟล์ต้องได้รับการอนุมัติ",
  "chat.welcomeSub": "เอไอเขียนโค้ดที่เร็วและเข้าใจโปรเจกต์ของคุณ",
  "chat.noProvider": "ยังไม่ได้ตั้งค่าผู้ให้บริการ — เปิดตั้งค่าเพื่อใส่ API key",
  "chat.thinking": "ระดับความคิด",
  "chat.context": "บริบท",
  "chat.sendKey": "เพื่อส่ง",
  "chat.newlineKey": "เพื่อขึ้นบรรทัดใหม่",
  "thinking.auto": "อัตโนมัติ",
  "thinking.off": "ปิด",
  "thinking.low": "ต่ำ",
  "thinking.medium": "กลาง",
  "thinking.high": "สูง",
  "thinking.auto.hint": "ให้โมเดลเลือกความลึกในการคิดเอง",
  "thinking.off.hint": "เร็วที่สุด — ไม่คิดเชิงลึกเพิ่มเติม",
  "thinking.low.hint": "คิดเล็กน้อยสําหรับงานง่าย",
  "thinking.medium.hint": "คิดแบบสมดุลสําหรับงานทั่วไป",
  "thinking.high.hint": "คิดลึกและละเอียดสําหรับงานยาก",
  "approval.required": "ต้องการการอนุมัติ",
  "approval.wantModify": "Volt ต้องการแก้ไขไฟล์ในโปรเจกต์ของคุณ",
  "approval.wantRun": "Volt ต้องการรันคำสั่งในโปรเจกต์ของคุณ",
  "approval.approve": "อนุมัติ",
  "approval.deny": "ปฏิเสธ",
  "settings.title": "ตั้งค่า",
  "settings.providers": "ผู้ให้บริการ",
  "settings.appearance": "ธีมและรูปลักษณ์",
  "settings.language": "ภาษา",
  "settings.general": "ทั่วไป",
  "settings.mcp": "เซิร์ฟเวอร์ MCP",
  "settings.plugins": "สกิลและปลั๊กอิน",
  "settings.about": "เกี่ยวกับ",
  "settings.addProvider": "เพิ่มผู้ให้บริการ",
  "settings.noProviders": "ยังไม่มีผู้ให้บริการ เพิ่มเพื่อเริ่มใช้งาน Volt",
  "settings.active": "ใช้งานอยู่",
  "settings.use": "ใช้",
  "settings.editProvider": "แก้ไขผู้ให้บริการ",
  "settings.type": "ประเภท",
  "settings.name": "ชื่อที่แสดง",
  "settings.baseUrl": "Base URL",
  "settings.model": "โมเดล",
  "settings.apiKey": "API key",
  "settings.keyHint": "จัดเก็บอย่างปลอดภัยใน OS keyring — ไม่เขียนลงดิสก์แบบข้อความธรรมดา",
  "settings.useBearer": "ส่ง Authorization: Bearer header",
  "settings.test": "ทดสอบการเชื่อมต่อ",
  "settings.save": "บันทึกผู้ให้บริการ",
  "settings.back": "ย้อนกลับ",
  "theme.dark": "มืด",
  "theme.light": "สว่าง",
  "theme.system": "ตามระบบ",
  "theme.accent": "สีหลัก",
  "general.approval": "นโยบายการอนุมัติ",
  "general.approvalWrite": "ถามก่อนเขียนไฟล์",
  "general.approvalRun": "ถามก่อนรันคำสั่ง",
  "general.approvalRead": "ถามก่อนอ่านไฟล์",
  "general.compaction": "การย่อบริบท",
  "general.compactionEnable": "เปิดการย่อบริบทอัตโนมัติ",
  "general.compactionThreshold": "ย่อเมื่อใช้ถึง",
  "general.maxIterations": "จำนวนรอบสูงสุดของเอเจนต์",
  "general.autoUpdate": "ตรวจหาอัปเดตเมื่อเปิดโปรแกรม",
  "mcp.add": "เพิ่มเซิร์ฟเวอร์",
  "mcp.empty": "ยังไม่มีเซิร์ฟเวอร์ MCP เชื่อมต่อเครื่องมืออย่าง filesystem, git หรือ fetch",
  "mcp.connect": "เชื่อมต่อ",
  "mcp.disconnect": "ตัดการเชื่อมต่อ",
  "mcp.connected": "เชื่อมต่อแล้ว",
  "mcp.transport": "รูปแบบการเชื่อมต่อ",
  "mcp.command": "คำสั่ง",
  "mcp.url": "URL",
  "plugins.add": "เพิ่มสกิล",
  "plugins.empty": "ยังไม่มีสกิล เพิ่มสกิลคำสั่งเองเพื่อขยายความสามารถของ Volt",
  "plugins.prompt": "คำสั่งพรอมต์",
  "plugins.command": "คำสั่งสแลช",
  "about.version": "เวอร์ชัน",
  "about.repo": "ที่เก็บ GitHub",
  "about.checkUpdates": "ตรวจหาอัปเดต",
  "about.upToDate": "คุณใช้เวอร์ชันล่าสุดแล้ว",
  "about.updateAvailable": "มีอัปเดตใหม่",
  "about.download": "ดาวน์โหลดและติดตั้ง",
  "about.downloading": "กำลังดาวน์โหลด…",
  "update.newVersion": "Volt {version} พร้อมให้อัปเดตแล้ว",
  "update.view": "ดู",
  "update.later": "ไว้ทีหลัง",
  "canvas.title": "แผนงาน",
  "canvas.empty": "ไม่มีแผนงานที่ใช้งานอยู่ ลองให้ Volt วางแผนงานหลายขั้นตอน",
  "canvas.pending": "รอดำเนินการ",
  "canvas.inProgress": "กำลังทำ",
  "canvas.completed": "เสร็จแล้ว",
  "canvas.compacted": "ย่อบริบทแล้ว",
};

const dictionaries: Record<Lang, Dict> = { en, th };

/** Translate a key, with optional `{placeholder}` interpolation. */
export function translate(
  lang: Lang,
  key: string,
  vars?: Record<string, string | number>,
): string {
  const dict = dictionaries[lang] ?? en;
  let text = dict[key] ?? en[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
    }
  }
  return text;
}

/** React hook — returns a `t()` bound to the current language. */
export function useT() {
  const language = useSettingsStore((s) => s.language) as Lang;
  return (key: string, vars?: Record<string, string | number>) =>
    translate(language, key, vars);
}

export const LANGUAGES: { id: Lang; label: string; flag: string }[] = [
  { id: "en", label: "English", flag: "🇬🇧" },
  { id: "th", label: "ไทย", flag: "🇹🇭" },
];
