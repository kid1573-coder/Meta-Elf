const fs = require('fs');
const p = "D:\\Work_Ai\\多语言翻译.html";
let code = fs.readFileSync(p, 'utf8');

// 1. Replace DOM structure
const domOld = `      <div class="grid">
        <div class="panel">
          <div class="panelHeader">
            <div class="currentTitle">选择</div>
          </div>
          <div class="panelBody">
            <div class="sidebarTitle">文章</div>
            <div class="list" id="articleList"></div>
            <div class="sidebarTitle" style="margin-top: 14px">语言</div>
            <div class="list" id="langList"></div>
          </div>
        </div>

        <div class="panel">
          <div class="panelHeader">`;
const domNew = `      <div class="grid">
        <div class="panel filter-panel">
          <div class="panelBody" style="padding: 16px 20px;">
            <div class="filter-row">
              <div class="sidebarTitle">文章</div>
              <div class="list" id="articleList"></div>
            </div>
            <div class="filter-row" style="margin-top: 16px;">
              <div class="sidebarTitle">语言</div>
              <div class="list" id="langList"></div>
            </div>
          </div>
        </div>

        <div class="panel main-panel">
          <div class="panelHeader sticky-header">`;

if (code.includes('class="sidebarTitle">文章</div>')) {
  // If the old DOM is present, replace it. 
  // Use a regex that matches the structure just in case of formatting diffs.
  code = code.replace(/<div class="grid">[\s\S]*?<div class="panel">[\s\S]*?<div class="panelHeader">/m, domNew);
}

// 2. Replace grid CSS
const gridOld = `.grid {
        display: grid;
        grid-template-columns: 280px 1fr;
        gap: 12px;
      }
      @media (max-width: 980px) {
        .grid {
          grid-template-columns: 1fr;
        }
      }`;
const gridNew = `.grid {
        display: flex;
        flex-direction: column;
        gap: 16px;
      }`;
if(code.includes('grid-template-columns: 280px 1fr;')) {
  code = code.replace(gridOld, gridNew);
} else {
  // Regex fallback
  code = code.replace(/\.grid\s*\{[\s\S]*?@media\s*\(max-width:\s*980px\)\s*\{[\s\S]*?\}/, gridNew);
}

// 3. Replace list/itemBtn CSS
const listCssOldRegex = /\.sidebarTitle\s*\{[\s\S]*?\.articleLabel\s*\{[\s\S]*?\}/;
const listCssNew = `.sticky-header {
        position: sticky;
        top: 0;
        z-index: 100;
        background: rgba(15, 23, 48, 0.95);
        backdrop-filter: blur(10px);
        border-bottom: 1px solid var(--line);
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
      }
      .filter-row {
        display: flex;
        align-items: flex-start;
        gap: 12px;
      }
      .sidebarTitle {
        font-size: 14px;
        color: var(--text);
        font-weight: 600;
        margin: 0;
        white-space: nowrap;
        padding-top: 8px;
        width: 40px;
      }
      .list {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: 10px;
        max-height: 220px;
        overflow-y: auto;
        padding-right: 4px;
        flex: 1;
      }
      /* 滚动条美化 */
      .list::-webkit-scrollbar {
        width: 6px;
      }
      .list::-webkit-scrollbar-thumb {
        background: rgba(255,255,255,0.15);
        border-radius: 4px;
      }
      .itemBtn {
        cursor: pointer;
        border: 1px solid var(--line);
        background: rgba(43, 63, 122, 0.12);
        color: var(--text);
        padding: 6px 14px;
        border-radius: 999px;
        font-size: 13px;
        display: inline-flex;
        align-items: center;
        gap: 8px;
        white-space: nowrap;
        transition: all 0.2s ease;
      }
      .itemBtn:hover {
        background: rgba(43, 63, 122, 0.3);
      }
      .itemBtn[aria-selected="true"] {
        border-color: rgba(110, 168, 255, 0.6);
        background: rgba(110, 168, 255, 0.2);
        color: #fff;
        font-weight: 500;
        box-shadow: 0 0 10px rgba(110, 168, 255, 0.15);
      }
      .articleItemBtn {
        border-radius: 8px;
        padding: 6px 12px;
      }
      .articleLabel {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 8px;
      }`;
code = code.replace(/\.sidebarTitle\s*\{[\s\S]*?\.articleLabel\s*\{[\s\S]*?\}/, listCssNew);

// 4. Update textarea min-height
code = code.replace(/min-height:\s*250px;/, 'min-height: 450px;');

// 5. Update preview max-height (remove it so it can expand)
code = code.replace(/max-height:\s*560px;/, 'min-height: 450px;');

// Add HTML smooth scrolling
if(!code.includes('scroll-behavior: smooth')) {
  code = code.replace('body {', 'html { scroll-behavior: smooth; }\n      body {');
}

fs.writeFileSync(p, code);
console.log("Updated HTML successfully!");