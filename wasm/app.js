import init, { DinerDashApp } from './pkg/diner_dash_wasm.js';

const $ = id => document.getElementById(id);
let app, view = 'menu', selected = { stage: 1, level: 1 }, muted = false;
let manifest, layouts, status, menuAnims = [], comicState = null, inCanvasPauseBtn = null;
const dialog = $('settings-dialog');

async function fetchData(path, type = 'json') {
  const response = await fetch(path);
  if (!response.ok) throw new Error(`Unable to load ${path} (${response.status})`);
  return response[type]();
}

// Read visual records (including multi-frame animations) from the extracted Lingo layout.
export function readLayout(text) {
  const nodes = [];
  for (const m of text.matchAll(/#frames: \[/g)) {
    const prefix = text.slice(0, m.index);
    const header = prefix.slice(prefix.lastIndexOf('#nodeID:'));
    const id = header.match(/#nodeID: "([^"]*)"/)?.[1] ?? '';
    const endIdx = text.indexOf('], #children:', m.index);
    const framesBlock = text.slice(m.index, (endIdx > m.index ? endIdx : text.indexOf(']', m.index)) + 1);
    const frames = [];
    for (const fm of framesBlock.matchAll(/\[#[^\]]+\]/g)) {
      const f = fm[0];
      const member = f.match(/#member: "([^"]+)"/)?.[1];
      const cast = f.match(/#cast: "([^"]+)"/)?.[1];
      const rect = f.match(/#rect: rect\(([^)]+)\)/)?.[1].split(',').map(Number);
      const z = Number(f.match(/#z: (-?\d+)/)?.[1] ?? 0);
      const anim = f.match(/#animID: "([^"]*)"/)?.[1] ?? '';
      if (member && cast && rect?.length === 4 && rect.every(Number.isFinite)) {
        frames.push({ key: `${cast}/${member}`, rect, z, anim });
      }
    }
    if (frames.length) {
      nodes.push({ id, key: frames[0].key, rect: frames[0].rect, z: frames[0].z, anim: frames[0].anim, frames });
    }
  }
  return nodes.sort((a, b) => a.z - b.z);
}

function place(element, rect) {
  const [x, y, right, bottom] = rect;
  Object.assign(element.style, { left: `${x / 8}%`, top: `${y / 6}%`,
    width: `${(right - x) / 8}%`, height: `${(bottom - y) / 6}%` });
}

function sfx(name) { try { app?.play_sfx(name); } catch {} }
function syncMusic() {
  try {
    if (!app) return;
    if (view === 'play' || view === 'prelevel') app.ensure_stage_music(status?.stage || selected.stage || 1);
    else app.ensure_menu_music();
  } catch {}
}

function art(node, callback, label, key = node.key, hover) {
  if (!manifest[key]) return;
  const el = document.createElement(callback ? 'button' : 'img');
  el.className = callback ? 'art-button' : 'menu-art';
  place(el, node.rect);
  el.style.zIndex = String(node.z);
  if (callback) {
    el.setAttribute('aria-label', label);
    const img = document.createElement('img');
    img.src = manifest[key].path; img.alt = '';
    el.append(img);
    el.onclick = () => { sfx('select'); safely(callback); };
    el.onpointerenter = () => { if (manifest[hover]) img.src = manifest[hover].path; sfx('rollover'); };
    el.onpointerleave = () => { if (manifest[hover]) img.src = manifest[key].path; };
    el.onfocus = el.onpointerenter; el.onblur = el.onpointerleave;
  } else {
    el.src = manifest[key].path; el.alt = '';
    if (node.frames?.length > 1) {
      for (const f of node.frames) {
        if (manifest[f.key]) { const pre = new Image(); pre.src = manifest[f.key].path; }
      }
      menuAnims.push({ el, node, idx: 0 });
    }
  }
  $('screen').append(el);
  return el;
}

function refresh() { status = JSON.parse(app.get_status_json()); }

function unlocked(stage, level) {
  const p = status.career_save;
  const frontier = Math.max((p.furthest_stage || 1) * 10 + (p.furthest_level || 1),
    p.current_stage * 10 + p.current_level);
  return stage * 10 + level <= frontier;
}

function ensureInCanvasPauseButton() {
  if (inCanvasPauseBtn || !manifest?.['gfx.screens/pauseButton_off']) return;
  const btn = document.createElement('button');
  btn.className = 'in-canvas-pause';
  btn.setAttribute('aria-label', 'Pause');
  const img = document.createElement('img');
  img.src = manifest['gfx.screens/pauseButton_off'].path;
  img.alt = 'PAUSE';
  btn.append(img);
  btn.onpointerenter = () => {
    if (manifest['gfx.screens/pauseButton_on']) img.src = manifest['gfx.screens/pauseButton_on'].path;
    sfx('rollover');
  };
  btn.onpointerleave = () => {
    if (manifest['gfx.screens/pauseButton_off']) img.src = manifest['gfx.screens/pauseButton_off'].path;
  };
  btn.onclick = () => { sfx('select'); safely(() => show('paused')); };
  $('stage').append(btn);
  inCanvasPauseBtn = btn;
}

function startComic(pages, onDone) {
  comicState = { pages, idx: 0, onDone };
  show('comic');
}

function beginCareerShift(stage, level) {
  selected = { stage, level };
  const launchPrelevel = () => {
    app.start_career_level(stage, level);
    app.set_paused(false);
    app.tick(0);
    app.set_paused(true);
    show('prelevel');
  };
  if (stage === 1 && level === 1) {
    startComic(['gfx.screens/intro-01-color', 'gfx.screens/intro-02-color', 'gfx.screens/intro-03-color'], launchPrelevel);
  } else if (stage === 5 && level === 1) {
    startComic(['gfx.screens/transformation-01-color', 'gfx.screens/transformation-02-colorB'], launchPrelevel);
  } else {
    launchPrelevel();
  }
}

let helpPage = 1;

function show(next) {
  view = next;
  app.set_paused(next !== 'play');
  refresh();
  syncMusic();
  ensureInCanvasPauseButton();
  menuAnims = [];
  $('screen').replaceChildren();
  $('panel').hidden = true;
  $('screen').hidden = next === 'play';
  $('level-list').hidden = next !== 'career';
  $('pause').hidden = next !== 'play';
  if (inCanvasPauseBtn) inCanvasPauseBtn.hidden = next !== 'play' && next !== 'prelevel';
  $('resume').hidden = next === 'play' || next === 'prelevel' || !!status.level_complete || !status.has_session;
  if (next === 'menu') renderMenu();
  if (next === 'help') renderHelp();
  if (next === 'career') renderCareer();
  if (next === 'endless') renderEndless();
  if (next === 'comic') renderComic();
  if (next === 'prelevel') renderPrelevel();
  if (next === 'play') {
    $('shift-label').textContent = `${status.stages[status.stage - 1]?.name} · Shift ${status.level}`;
    $('hint').textContent = 'Select a waiting party → clean table. Take orders when menus are ready. Kitchen → serve food. Check → clean table. Esc pauses.';
  }
  if (next === 'paused') {
    $('screen').hidden = true;
    panel('Shift paused', 'Take a breather. Your shift is saved.', [
      ['Resume', () => show('play')], ['Main menu', () => show('menu')],
    ]);
  }
}

function renderHelp() {
  const isCredits = helpPage === 'credits';
  $('shift-label').textContent = isCredits ? 'Diner Dash · Credits' : `How to Play · Page ${helpPage} of 5`;
  $('hint').textContent = 'Use the arrows to flip through the instruction pages, or return to the main menu.';

  art({ id: 'help_frame', key: 'gfx.screens/help_frame', rect: [0, 0, 800, 600], z: 1 });

  const pageSpecs = {
    1: { key: 'gfx.screens/help_page_1', rect: [80, 67, 709, 529] },
    2: { key: 'gfx.screens/help_page_2', rect: [72, 66, 728, 549] },
    3: { key: 'gfx.screens/help_page_3', rect: [72, 67, 715, 541] },
    4: { key: 'gfx.screens/help_page_4', rect: [72, 49, 714, 510] },
    5: { key: 'gfx.screens/help_page_5', rect: [73, 68, 716, 534] },
    credits: { key: 'gfx.screens/help_CREDITS_PAGE', rect: [67, 49, 728, 543] },
  };
  const spec = pageSpecs[helpPage] || pageSpecs[1];
  art({ id: 'page', key: spec.key, rect: spec.rect, z: 3 });

  if (isCredits) {
    art({ id: 'title', key: 'gfx.screens/help_title_CREDITS', rect: [331, 3, 474, 44], z: 4 });
    art(
      { id: 'instructions_btn', key: 'gfx.screens/help_button_instruction', rect: [416, 542, 593, 586], z: 6 },
      () => { helpPage = 1; show('help'); },
      'Instructions',
      'gfx.screens/help_button_instruction',
      'gfx.screens/help_button_instruction_up'
    );
  } else {
    art({ id: 'title', key: 'gfx.screens/help_title_instructions', rect: [282, 2, 517, 43], z: 4 });
    art(
      { id: 'credits_btn', key: 'gfx.screens/help_button_credits', rect: [416, 542, 593, 586], z: 6 },
      () => { helpPage = 'credits'; show('help'); },
      'Credits',
      'gfx.screens/help_button_credits',
      'gfx.screens/help_button_credits_up'
    );
  }

  art(
    { id: 'menubutton', key: 'gfx.screens/help_button_backtomenu', rect: [206, 541, 383, 586], z: 10 },
    () => show('menu'),
    'Back to Menu',
    'gfx.screens/help_button_backtomenu',
    'gfx.screens/help_button_backtomenu_up'
  );

  if (!isCredits && helpPage > 1) {
    art(
      { id: 'previous', key: 'gfx.screens/help_arrow', rect: [21, 523, 120, 584], z: 12 },
      () => { helpPage -= 1; show('help'); },
      'Previous page',
      'gfx.screens/help_arrow',
      'gfx.screens/help_arrow_up'
    );
  }
  if (!isCredits && helpPage < 5) {
    const nextBtn = art(
      { id: 'next', key: 'gfx.screens/help_arrow', rect: [673, 524, 773, 584], z: 11 },
      () => { helpPage += 1; show('help'); },
      'Next page',
      'gfx.screens/help_arrow',
      'gfx.screens/help_arrow_up'
    );
    if (nextBtn) nextBtn.style.transform = 'scaleX(-1)';
  }
}

function renderComic() {
  if (!comicState) { show('career'); return; }
  const { pages, idx, onDone } = comicState;
  const pageKey = pages[idx];
  if (pageKey?.includes('transformation-02')) sfx('goddess');
  $('shift-label').textContent = 'Flo’s Story';
  $('hint').textContent = `Comic page ${idx + 1} of ${pages.length} · Click the arrow in the bottom-right corner to continue.`;
  art({ id: 'bg', key: 'gfx.screens/help_frame', rect: [0, 0, 800, 600], z: 1 });
  art({ id: 'page', key: pageKey, rect: [80, 60, 720, 540], z: 3 });
  if (idx > 0) {
    art(
      { id: 'prev_button', key: 'gfx.screens/help_arrow_up', rect: [28, 537, 127, 598], z: 5 },
      () => { comicState.idx -= 1; show('comic'); },
      'Previous page',
      'gfx.screens/help_arrow_up',
      'gfx.screens/help_arrow'
    );
  }
  const nextBtn = art(
    { id: 'next_button', key: 'gfx.screens/help_arrow_up', rect: [675, 537, 774, 598], z: 6 },
    () => {
      if (comicState.idx + 1 < comicState.pages.length) {
        comicState.idx += 1;
        show('comic');
      } else {
        const finish = onDone;
        comicState = null;
        finish();
      }
    },
    'Next page',
    'gfx.screens/help_arrow_up',
    'gfx.screens/help_arrow'
  );
  if (nextBtn) nextBtn.style.transform = 'scaleX(-1)';
}

function levelDoodleSpec(stage, level) {
  const key = `${stage}_${level}`;
  const table = {
    '1_1': { src: 'assets/images/leveltext/younggirl_63.png', rect: [582, 156, 659, 383], textRect: [200, 113, 578, 441] },
    '1_6': { src: 'assets/images/leveltext/foodcritic_65.png', rect: [578, 146, 671, 386], textRect: [200, 113, 574, 441] },
    '1_7': { src: 'assets/images/leveltext/oldman_66.png', rect: [589, 123, 671, 352], textRect: [200, 113, 584, 441] },
    '2_6': { src: 'assets/images/leveltext/business_woman_191.png', rect: [588, 108, 671, 354], textRect: [200, 113, 584, 441] },
    '3_7': { src: 'assets/images/leveltext/youngboy_67.png', rect: [577, 116, 661, 356], textRect: [189, 99, 573, 427] },
    '4_7': { src: 'assets/images/leveltext/bizman_64.png', rect: [583, 117, 666, 355], textRect: [200, 113, 579, 441] },
  };
  return table[key] || { src: null, rect: null, textRect: [200, 113, 665, 441] };
}

async function renderPrelevel() {
  const stage = status.stage || selected.stage || 1;
  const level = status.level || selected.level || 1;
  $('shift-label').textContent = `${status.stages[stage - 1]?.name} · Shift ${level}`;
  $('hint').textContent = 'Read the daily special note on the chalkboard, then click LET’S PLAY!';

  // 50% dim overlay over the restaurant stage (mousetrap in startlevel_layout.txt)
  const dimEl = document.createElement('div');
  dimEl.className = 'prelevel-dim';
  $('screen').append(dimEl);

  art({ id: 'bg', key: 'gfx.screens/endshop_bg', rect: [65, 26, 700, 567], z: 1 });

  const spec = levelDoodleSpec(stage, level);
  const textEl = document.createElement('div');
  textEl.className = 'chalkboard-text';
  place(textEl, spec.textRect);
  textEl.style.zIndex = '2';
  textEl.textContent = 'WELCOME TO FLO’S DINER!';
  $('screen').append(textEl);

  if (spec.src && spec.rect) {
    const doodleEl = document.createElement('img');
    doodleEl.className = 'menu-art';
    place(doodleEl, spec.rect);
    doodleEl.style.zIndex = '3';
    doodleEl.src = spec.src;
    doodleEl.alt = '';
    $('screen').append(doodleEl);
  }

  art(
    { id: 'ok_button', key: 'gfx.screens/lets_play_normal', rect: [315, 451, 519, 509], z: 7 },
    () => show('play'),
    "Let's Play!",
    'gfx.screens/lets_play_normal',
    'gfx.screens/lets_play_rollover'
  );

  try {
    const file = status.mode === 'Endless'
      ? 'assets/data/leveltext/startlvl_text_endless.txt'
      : `assets/data/leveltext/startlvl_text_${stage}-${level}.txt`;
    const raw = await fetchData(file, 'text');
    if (view !== 'prelevel') return;
    const cleaned = raw.replace(/\r\n/g, '\n').trim();
    textEl.textContent = cleaned;
  } catch {}
}

function renderMenu() {
  $('shift-label').textContent = 'Welcome to Flo’s Diner';
  $('hint').textContent = 'Choose Career to select a restaurant and shift, or play Endless mode.';
  const actions = {
    career_button: () => {
      selected = { stage: status.career_save.current_stage, level: status.career_save.current_level };
      show('career');
    },
    endless_button: () => show('endless'),
    help_button: () => { helpPage = 1; show('help'); },
    options_button: openSettings,
    hiscore_button: () => panel('Your diner', `Customers served: ${status.stats.total_customers_served}. Expert shifts: ${status.stats.expert_scores}.`, [['Back', () => show('menu')]]),
    quit_button: () => panel('Shift saved', 'You can safely close this tab.', [['Back', () => show('menu')]]),
  };
  for (const node of layouts.title) {
    if (['login_button', 'username_txt', 'user_bg'].includes(node.id)) continue;
    const action = actions[node.id];
    art(node, action, node.id.replace('_button', ''), node.key, `${node.key}_on`);
  }
}

function renderCareer() {
  $('shift-label').textContent = 'Flo’s career · 5 restaurants, 50 shifts';
  const stage = status.stages[selected.stage - 1];
  $('hint').textContent = `${stage?.name} · Shift ${selected.level}. Choose a dot, then Start Shift. Stars mark Expert scores.`;
  for (const node of layouts.career) {
    if (/^(const_|const_anim_|statue_[12]|fadeout|sparkle_|lockall_|unlockall_)/.test(node.id)) continue;
    if (node.id === 'menu_button') { art(node, () => show('menu'), 'Main menu'); continue; }
    const canContinue = status.has_session && !status.level_complete
      && status.mode === 'Career' && selected.stage === status.stage && selected.level === status.level;
    if (node.id === 'play_button') {
      if (canContinue) art(node, () => show('play'), 'Continue saved shift', node.key, 'gfx.screens/button_continue_on');
      continue;
    }
    if (node.id === 'newgame_button') {
      if (!canContinue) art(node, () => beginCareerShift(selected.stage, selected.level), 'Start Shift', node.key, 'gfx.screens/button_new_game_up');
      continue;
    }
    const level = node.id.match(/^lvlbutton_(\d+)_(\d+)$/);
    if (level) {
      const s = Number(level[1]), l = Number(level[2]);
      const chosen = selected.stage === s && selected.level === l;
      const isContinue = status.has_session && !status.level_complete
        && status.mode === 'Career' && s === status.stage && l === status.level;
      const expert = status.career_save.level_expert[`${s}_${l}`];

      let key = 'gfx.screens/leveldot_normal';
      if (isContinue) {
        key = chosen ? 'gfx.screens/lvlcontinue_selected' : 'gfx.screens/lvlcontinue_normal';
      } else if (expert) {
        key = chosen ? 'gfx.screens/leveldot_star_selected' : 'gfx.screens/leveldot_star';
      } else {
        key = chosen ? 'gfx.screens/leveldot_selected' : 'gfx.screens/leveldot_normal';
      }

      const button = art({ ...node, z: 100 },
        () => { selected = { stage: s, level: l }; show('career'); },
        `${status.stages[s - 1].name}, shift ${l}${expert ? ', Expert' : ''}`, key);
      if (button) {
        button.classList.add('level-dot'); button.disabled = !unlocked(s, l);
        button.setAttribute('aria-pressed', String(chosen));
      }
      continue;
    }
    if (/^rest_[2-5]$/.test(node.id) && !unlocked(Number(node.id.slice(-1)), 1)) continue;
    if (node.id === 'statue_cover' && !unlocked(5, 1)) continue;
    art(node);
  }
  const list = $('level-list'); list.replaceChildren();
  const details = document.createElement('details');
  const summary = document.createElement('summary'); summary.textContent = 'Restaurant & shift list';
  summary.onpointerenter = () => sfx('rollover'); summary.onclick = () => sfx('select');
  details.append(summary);
  status.stages.forEach((stage, idx) => {
    const heading = document.createElement('h3'); heading.textContent = stage.name; details.append(heading);
    const row = document.createElement('div'); row.className = 'level-row';
    for (let l = 1; l <= stage.num_levels; l++) {
      const button = document.createElement('button'); button.textContent = String(l);
      button.disabled = !unlocked(idx + 1, l);
      button.onpointerenter = () => { if (!button.disabled) sfx('rollover'); };
      button.onclick = () => { sfx('select'); selected = { stage: idx + 1, level: l }; show('career'); };
      row.append(button);
    }
    details.append(row);
  });
  list.append(details);
}

function renderEndless() {
  $('shift-label').textContent = 'Endless Mode · Serve as long as you can';
  $('hint').textContent = 'Choose an unlocked restaurant, then select difficulty to begin.';

  // Background (`endless_setup_layout.txt`)
  art({ id: 'bg', key: 'gfx.screens/background', rect: [0, 0, 800, 600], z: 1 });
  // Title
  art({ id: 'title', key: 'gfx.screens/Title_Endless', rect: [5, 0, 535, 69], z: 17 });
  // Menu button
  art({ id: 'menu_button', key: 'gfx.screens/button_back_to_menu', rect: [686, 6, 797, 34], z: 18 },
    () => show('menu'), 'Main menu', 'gfx.screens/button_back_to_menu', 'gfx.screens/button_back_to_menu_on');

  // Restaurant buildings
  const restConfigs = [
    { stage: 1, key: 'gfx.screens/r1', rect: [53, 105, 305, 283], z: 2 },
    { stage: 2, key: 'gfx.screens/RT_2', rect: [486, 57, 789, 330], z: 7 },
    { stage: 3, key: 'gfx.screens/RT_3', rect: [591, 337, 800, 590], z: 5 },
    { stage: 4, key: 'gfx.screens/RT_4', rect: [0, 318, 321, 600], z: 3 },
    { stage: 5, key: 'gfx.screens/RT_5', rect: [279, 225, 524, 398], z: 11 },
  ];

  for (const rc of restConfigs) {
    const isUnlocked = unlocked(rc.stage, 1);
    if (!isUnlocked && rc.stage > 1) continue;
    art({ id: `rest_${rc.stage}`, key: rc.key, rect: rc.rect, z: rc.z },
      () => openEndlessDifficulty(rc.stage),
      status.stages[rc.stage - 1]?.name || `Restaurant ${rc.stage}`);
  }

  if (!unlocked(5, 1)) {
    art({ id: 'statue_cover', key: 'gfx.screens/R5_statue_blocker', rect: [340, 219, 442, 306], z: 10 });
  }
}

function openEndlessDifficulty(stageNum) {
  const diffEl = document.createElement('div');
  diffEl.className = 'endless-dialog-container';
  diffEl.id = 'endless-diff-container';

  // Dim overlay
  const trap = document.createElement('div');
  trap.className = 'endless-mousetrap';
  diffEl.append(trap);

  const addDiffArt = (key, rect, z) => {
    if (!manifest[key]) return null;
    const img = document.createElement('img');
    img.src = manifest[key].path;
    img.className = 'menu-art';
    img.style.zIndex = String(z);
    place(img, rect);
    diffEl.append(img);
    return img;
  };

  // Window hsp_wind at [175, 137, 638, 489], z 201
  addDiffArt('gfx.screens/hsp_wind', [175, 137, 638, 489], 201);
  // Title ENDLESS_MSG at [262, 271, 544, 326], z 202
  addDiffArt('gfx.screens/ENDLESS_MSG', [262, 271, 544, 326], 202);

  // Difficulty buttons
  const diffs = [
    { diff: 1, rect: [206, 352, 319, 394], off: 'gfx.screens/diff_1_off', on: 'gfx.screens/diff_1_on', label: 'Easy' },
    { diff: 2, rect: [331, 353, 475, 393], off: 'gfx.screens/diff_2_off', on: 'gfx.screens/diff_2_on', label: 'Medium' },
    { diff: 3, rect: [486, 354, 599, 396], off: 'gfx.screens/diff_3_off', on: 'gfx.screens/diff_3_on', label: 'Hard' },
  ];

  for (const d of diffs) {
    const btn = document.createElement('button');
    btn.className = 'art-button';
    btn.style.zIndex = '203';
    btn.setAttribute('aria-label', d.label);
    place(btn, d.rect);
    const img = document.createElement('img');
    img.src = manifest[d.off]?.path || '';
    btn.append(img);
    btn.onpointerenter = () => {
      sfx('rollover');
      if (manifest[d.on]) img.src = manifest[d.on].path;
    };
    btn.onpointerleave = () => {
      if (manifest[d.off]) img.src = manifest[d.off].path;
    };
    btn.onclick = () => {
      sfx('select');
      diffEl.remove();
      safely(() => {
        app.start_endless_stage(stageNum, d.diff);
        show('play');
      });
    };
    diffEl.append(btn);
  }

  // Close button hsp_menu at [302, 412, 507, 451], z 204
  const closeBtn = document.createElement('button');
  closeBtn.className = 'art-button';
  closeBtn.style.zIndex = '204';
  closeBtn.setAttribute('aria-label', 'Close');
  place(closeBtn, [302, 412, 507, 451]);
  const closeImg = document.createElement('img');
  closeImg.src = manifest['gfx.screens/hsp_menu']?.path || '';
  closeBtn.append(closeImg);
  closeBtn.onpointerenter = () => {
    sfx('rollover');
    if (manifest['gfx.screens/hsp_menu_on']) closeImg.src = manifest['gfx.screens/hsp_menu_on'].path;
  };
  closeBtn.onpointerleave = () => {
    if (manifest['gfx.screens/hsp_menu']) closeImg.src = manifest['gfx.screens/hsp_menu'].path;
  };
  closeBtn.onclick = () => {
    sfx('select');
    diffEl.remove();
  };
  diffEl.append(closeBtn);

  $('screen').append(diffEl);
}

function renderEndlevel(r) {
  app.set_paused(true);
  const el = $('panel'); el.replaceChildren(); el.hidden = false;

  // Sound
  if (r.success) {
    sfx('win');
  } else {
    sfx('angry');
  }

  // 1. Dim background
  const dim = document.createElement('div');
  dim.className = 'endlevel-dim';
  el.append(dim);

  // Helper to place sprite by rect [left, top, right, bottom]
  const addArt = (key, rect, z, className = '') => {
    if (!manifest[key]) return null;
    const img = document.createElement('img');
    img.src = manifest[key].path;
    img.className = 'menu-art ' + className;
    img.style.zIndex = String(z);
    place(img, rect);
    el.append(img);
    return img;
  };

  // 2. Main window background: rect [259, 39, 737, 564], z 6
  addArt('gfx.screens/endGame_window', [259, 39, 737, 564], 6);

  // 3. Flo character art
  if (r.success) {
    addArt('gfx.screens/flo_win', [1, 14, 345, 581], 15);
  } else {
    addArt('gfx.screens/flo_lose', [113, 19, 327, 589], 16);
  }

  // 4. Expert star: rect [469, 16, 616, 181], z 35
  if (r.expert) {
    addArt('gfx.screens/expert_star', [469, 16, 616, 181], 35);
  }

  // 5. Doodle (if won and doodle exists)
  if (r.success && r.doodle) {
    addArt('doodles/noise', [547, 292, 714, 503], 32);
    const doodleKey = 'doodles/' + r.doodle;
    if (manifest[doodleKey]) {
      addArt(doodleKey, [560, 334, 701, 494], 33);
    }
  }

  // 6. Text overlays
  const addText = (text, rect, z, className) => {
    const div = document.createElement('div');
    div.className = 'endlevel-text ' + className;
    div.textContent = text;
    div.style.zIndex = String(z);
    place(div, rect);
    el.append(div);
    return div;
  };

  // Textbox (dialog message): rect [289, 230, 532, 370], z 20
  addText(r.text, [289, 230, 532, 370], 20, 'endlevel-textbox');

  // If Career mode, show served, left, and level
  if (status.mode === 'Career') {
    addText(String(r.customers_served), [288, 195, 357, 222], 21, 'endlevel-metric');
    addText(String(r.customers_left), [373, 195, 442, 222], 22, 'endlevel-metric');
    addText(`${r.stage}-${r.level}`, [461, 192, 524, 219], 23, 'endlevel-metric');
  }

  // Scores
  addText(Number(r.score).toLocaleString(), [434, 443, 526, 464], 25, 'endlevel-score');
  const total = status.total_score || r.score;
  addText(Number(total).toLocaleString(), [392, 487, 528, 508], 27, 'endlevel-score');

  // 7. Action Button: rect [299, 384, 500, 441], z 40
  const nextAction = () => {
    app.next_shift();
    refresh();
    beginCareerShift(status.stage, status.level);
  };
  const endAction = () => {
    if (r.success) {
      startComic(['gfx.screens/end-01-color', 'gfx.screens/end-02-color', 'gfx.screens/end-03-color'], () => show('career'));
    } else {
      show('career');
    }
  };

  let btnKey = 'gfx.screens/continue_normal';
  let btnHover = 'gfx.screens/continue_rollover';
  let btnAction = nextAction;
  let btnAria = 'Continue to next shift';

  if (r.end_game) {
    btnKey = 'gfx.screens/quit_normal';
    btnHover = 'gfx.screens/quit_rollover';
    btnAction = endAction;
    btnAria = 'Quit';
  } else if (!r.success) {
    btnKey = 'gfx.screens/try_again_normal';
    btnHover = 'gfx.screens/try_again_rollover';
    btnAction = nextAction;
    btnAria = 'Try again';
  }

  const btn = document.createElement('button');
  btn.className = 'art-button endlevel-button';
  btn.style.zIndex = '40';
  btn.setAttribute('aria-label', btnAria);
  place(btn, [299, 384, 500, 441]);

  const img = document.createElement('img');
  img.src = manifest[btnKey]?.path || '';
  btn.append(img);

  btn.onpointerenter = () => {
    sfx('rollover');
    if (manifest[btnHover]) img.src = manifest[btnHover].path;
  };
  btn.onpointerleave = () => {
    if (manifest[btnKey]) img.src = manifest[btnKey].path;
  };
  btn.onclick = () => {
    sfx('select');
    el.hidden = true;
    el.replaceChildren();
    safely(btnAction);
  };

  el.append(btn);
  btn.focus();
}

function panel(title, text, actions) {
  app.set_paused(true);
  const el = $('panel'); el.replaceChildren(); el.hidden = false;
  const card = document.createElement('div'); card.className = 'panel-card';
  const h = document.createElement('h2'); h.textContent = title;
  const p = document.createElement('p'); p.textContent = text;
  const buttons = document.createElement('div'); buttons.className = 'panel-actions';
  for (const [label, action] of actions) {
    const b = document.createElement('button'); b.textContent = label;
    b.onpointerenter = () => sfx('rollover');
    b.onclick = () => { sfx('select'); safely(action); };
    buttons.append(b);
  }
  card.append(h, p, buttons); el.append(card); buttons.querySelector('button')?.focus();
}

function safely(action) {
  try { action(); $('error').hidden = true; }
  catch (error) { $('error').textContent = String(error); $('error').hidden = false; }
}
function openSettings() { app.set_paused(true); dialog.showModal(); }
dialog.addEventListener('close', () => app.set_paused(view !== 'play'));
for (const id of ['close-settings', 'home', 'pause', 'resume', 'settings', 'mute', 'save', 'load', 'reset', 'export']) {
  const el = $(id);
  if (el) el.addEventListener('pointerenter', () => sfx('rollover'));
}
$('close-settings').onclick = () => { sfx('select'); dialog.close(); };
$('home').onclick = e => { e.preventDefault(); sfx('select'); safely(() => show('menu')); };
$('pause').onclick = () => { sfx('select'); safely(() => show('paused')); };
$('resume').onclick = () => { sfx('select'); safely(() => show('play')); };
$('settings').onclick = () => { sfx('select'); safely(openSettings); };
$('mute').onclick = () => safely(() => {
  muted = !muted; app.set_muted(muted); if (!muted) { sfx('select'); syncMusic(); }
  $('mute').textContent = muted ? 'Sound off' : 'Sound on'; $('mute').setAttribute('aria-pressed', String(muted));
});
$('save').onclick = () => { sfx('select'); safely(() => app.save_now()); };
$('load').onclick = () => { sfx('select'); safely(() => { app.load_now(); dialog.close(); show('play'); }); };
$('reset').onclick = () => { sfx('select'); safely(() => {
  if (!confirm('Reset all diner progress? Export a backup first.')) return;
  app.reset_save(); dialog.close(); show('menu');
}); };
$('export').onclick = () => { sfx('select'); safely(() => {
  const url = URL.createObjectURL(new Blob([app.export_save_json()], { type: 'application/json' }));
  const a = document.createElement('a'); a.href = url; a.download = 'diner-dash-save.json'; a.click(); URL.revokeObjectURL(url);
}); };
$('import').onchange = async e => {
  const file = e.target.files[0]; if (!file) return;
  const text = await file.text(); safely(() => { app.import_save_json(text); dialog.close(); show('menu'); }); e.target.value = '';
};
window.addEventListener('pointerdown', syncMusic, { passive: true });
window.addEventListener('click', syncMusic, { passive: true });
window.addEventListener('keydown', syncMusic, { passive: true });

async function preloadAudio() {
  const sfxFiles = {
    rollover: 'assets/audio/sfx_rollover_1_316.wav?v=3',
    select: 'assets/audio/sfx_food_ready_1_snd_252.wav?v=3',
    seat: 'assets/audio/sfx_seat_people_snd_243.wav?v=3',
    order: 'assets/audio/sfx_pencil_write_2_730.wav?v=3',
    deliver_order: 'assets/audio/sfx_deliver_order_1_snd_263.wav?v=3',
    serve: 'assets/audio/sfx_deliver_food_1_snd_265.wav?v=3',
    pickup: 'assets/audio/sfx_pickup_food_1_snd_244.wav?v=3',
    done_eating: 'assets/audio/sfx_done_eating_1_snd_338.wav?v=3',
    cash: 'assets/audio/sfx_bring_check_1_snd_270.wav?v=3',
    clean: 'assets/audio/sfx_clear_dishes_1_snd_269.wav?v=3',
    dish_dropoff: 'assets/audio/sfx_dish_dropoff_1_snd_259.wav?v=3',
    drink_pickup: 'assets/audio/sfx_get_drinks_1_snd_250.wav?v=3',
    drink_dropoff: 'assets/audio/sfx_dropoff_drinks_1_snd_254.wav?v=3',
    angry: 'assets/audio/sfx_lose_heart_1_238.wav?v=3',
    win: 'assets/audio/sfx_build_rest_331.wav?v=3',
    goddess: 'assets/audio/sfx_goddess_appear_42.wav?v=3',
    heart: 'assets/audio/sfx_gain_heart_1_247.wav?v=3',
    arrive: 'assets/audio/sfx_party_arrive_1_snd_245.wav?v=3',
    ready: 'assets/audio/sfx_food_ready_1_snd_252.wav?v=3',
  };
  const ctx = new (window.AudioContext || window.webkitAudioContext)();
  await Promise.all(Object.entries(sfxFiles).map(async ([name, url]) => {
    try {
      const res = await fetch(url);
      if (!res.ok) return;
      const buf = await ctx.decodeAudioData(await res.arrayBuffer());
      app.register_audio_buffer(name, buf);
    } catch {}
  }));
  try { await ctx.close(); } catch {}
}

async function run() {
  [manifest, ...layouts] = await Promise.all([
    fetchData('assets/data/sprite_manifest.json'),
    fetchData('assets/data/scene_layouts/title_screen_layout.txt', 'text'),
    fetchData('assets/data/scene_layouts/career_setup_layout.txt', 'text'),
  ]);
  layouts = { title: readLayout(layouts[0]), career: readLayout(layouts[1]) };
  await init(); app = new DinerDashApp('game-canvas'); window.dinerDashApp = app;
  preloadAudio();
  show('menu');
  const canvas = $('game-canvas');
  const toStage = e => {
    const rect = canvas.getBoundingClientRect();
    return [(e.clientX - rect.left) * 800 / rect.width, (e.clientY - rect.top) * 600 / rect.height];
  };
  const interactive = e => view === 'play' && !dialog.open && e.button === 0;
  canvas.addEventListener('pointerdown', e => {
    if (!interactive(e)) return;
    e.preventDefault();
    const [x, y] = toStage(e);
    canvas.setPointerCapture(e.pointerId);
    safely(() => app.pointer_down(x, y));
  });
  canvas.addEventListener('pointermove', e => {
    if (view !== 'play' || dialog.open || !canvas.hasPointerCapture(e.pointerId)) return;
    const [x, y] = toStage(e);
    safely(() => app.pointer_move(x, y));
  });
  const release = e => {
    if (!canvas.hasPointerCapture(e.pointerId)) return;
    canvas.releasePointerCapture(e.pointerId);
    const [x, y] = toStage(e);
    safely(() => app.pointer_up(x, y));
  };
  canvas.addEventListener('pointerup', release);
  canvas.addEventListener('pointercancel', e => {
    if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
    safely(() => app.pointer_up(-1000, -1000));
  });
  window.addEventListener('keydown', e => {
    if (dialog.open || ['INPUT', 'BUTTON', 'SELECT', 'TEXTAREA'].includes(document.activeElement.tagName)) return;
    if (e.key === 'Escape') { safely(() => show(view === 'play' ? 'paused' : view === 'paused' ? 'play' : 'menu')); return; }
    if (view !== 'play') return;
    if ([' ', 'Enter', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) e.preventDefault();
    if (!e.repeat) app.key_down(e.key);
  });
  window.addEventListener('keyup', e => app.key_up(e.key));
  document.addEventListener('visibilitychange', () => { if (document.hidden && view === 'play') show('paused'); });
  window.addEventListener('beforeunload', () => safely(() => app.save_now()));
  let last;
  function frame(ts) {
    const dt = last === undefined ? 0 : Math.min((ts - last) / 1000, .1); last = ts;
    if (view !== 'play' && menuAnims.length) {
      for (const a of menuAnims) {
        const frames = a.node.frames;
        let idx = 0;
        if (a.node.anim === 'blink') {
          const cycle = ts % 2200;
          idx = cycle < frames.length * 60 ? Math.floor(cycle / 60) : 0;
        } else {
          idx = Math.floor(ts / 80) % frames.length;
        }
        if (idx !== a.idx) {
          a.idx = idx;
          const f = frames[idx];
          if (manifest[f.key]) {
            a.el.src = manifest[f.key].path;
            place(a.el, f.rect);
          }
        }
      }
    }
    if (view === 'play' && !dialog.open) safely(() => {
      status = JSON.parse(app.tick(dt));
      if (status.level_complete) {
        view = 'results'; $('pause').hidden = true;
        if (inCanvasPauseBtn) inCanvasPauseBtn.hidden = true;
        renderEndlevel(status.level_complete);
      }
    });
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  if ('serviceWorker' in navigator) navigator.serviceWorker.register('./sw.js').catch(console.warn);
}
run().catch(error => { $('error').textContent = `Unable to start the game: ${error}`; $('error').hidden = false; });
