property mgr, name, node_mgr, game_mgr, gameNode, titleNode, node, buttons, levelDots, stagedata, selectedDot, selectedDotMem, save_mgr, savedGame, currentLevel, timer, toggle, toggleCount, restaurants, construction, const_anim, statueAnim, rest5dots, statueCover, initialized, sparkles, statues, dotmem, starMem, currentDotMem, fadeOut, maxStage, showStatueAnim
global glob

on new me, m
  mgr = m
  mgrs = m.glob.managers
  node_mgr = mgrs.node
  game_mgr = mgrs.game
  save_mgr = mgrs.savegame
  gameNode = node_mgr.getGroup("game")
  titleNode = node_mgr.getGroup("titles")
  name = #startstage
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  dotmem = member("leveldot_normal", "gfx.screens")
  starMem = member("leveldot_star", "gfx.screens")
  currentDotMem = member("lvlcontinue_selected", "gfx.screens")
  node = titleNode.newNode(getNodeData("career_setup"))
  node["unlockall_button"].destroy()
  node["lockall_button"].destroy()
  node["star_text"].destroy()
  node.setVisible(0)
  buttons = [:]
  fadeOut = node["fadeout"]
  mems = [#rollover: member("button_continue_on", "gfx.screens"), #normal: member("button_continue", "gfx.screens")]
  buttons[#continue] = newButton([#node: node["play_button"], #members: mems])
  buttons.continue.setVisible(0)
  node["menu_button"].setVisible(0)
  node["career_title"].setVisible(0)
  node["career_text"].setVisible(0)
  node["newgame_button"].setVisible(0)
  restaurants = []
  construction = []
  const_anim = []
  repeat with i = 2 to 4
    restaurants.add(node["rest_" & i])
    construction.add(node["const_anim_" & i])
    const_anim.add(node["const_" & i])
  end repeat
  restaurants.add(node["rest_5"])
  rest5dots = node["rest5_dots"]
  statueCover = node["statue_cover"]
  statues = [node["statue_1"], node["statue_2"]]
  statueAnim = script("statue_anim", "scripts.mode").new([#mgr: me, #statue1: statues[1], #statueCover: statueCover, #statue2: statues[2], #door: node["rest_5"]])
  call(#setAnimGroup, const_anim, #titles)
  selectedDotMem = member("leveldot_selected", "gfx.screens")
  levelDots = []
  repeat with i = 1 to 5
    lvl = []
    repeat with j = 1 to 10
      lvl.add(node["lvlbutton_" & i & "_" & j])
    end repeat
    levelDots.add(lvl)
  end repeat
  timer = script("timeout").new()
  timer.setGroup("titles")
  timer.setPeriod(500)
  sparkles = []
  repeat with i = 1 to 4
    n = node["sparkle_" & i]
    n.newNode(getNodeData("sparkle1"))
    sparkles.add(n)
  end repeat
  n = node["sparkle_5"]
  n.newNode(getNodeData("sparkle1"))
  sparkles.add(n)
  call(#setVisible, sparkles, 0)
  call(#setAnimGroup, sparkles, "titles")
end

on showConstruction me
  SndSFX("sfx_unlock_stage")
  timer.stop()
  indx = currentLevel.stage - 1
  if (indx < 1) or (indx > restaurants.count) then
    return 
  end if
  restaurants[indx].setVisible(0)
  construction[indx].setVisible(1)
  timer.setCallback([#target: me, #handler: #showConstAnim])
  timer.start()
end

on showConstAnim me
  timer.stop()
  indx = currentLevel.stage - 1
  if (indx < 1) or (indx > 3) then
    return 
  end if
  ca = const_anim[indx]
  ca.setVisible(1)
  ca.setAnimation("const", [#frameMS: 300, #loopCount: 3, #callback: [#target: me, #handler: #hideConstAnim]])
end

on hideConstAnim me
  indx = currentLevel.stage - 1
  construction[indx].setVisible(0)
  restaurants[indx].setVisible(1)
  const_anim[indx].setVisible(0)
  buttons.continue.setVisible(1)
end

on startStatueAnim me
  SndSFX("sfx_goddess_appear")
  restaurants[4].setZOffset(100)
  Args = [#node: fadeOut, #setter: #setAlpha, #start: 0, #end: 50, #ms: 500, #group: "titles"]
  script("node_tween").new(Args)
  statueAnim.start()
end

on statueDone me
  Args = [#node: fadeOut, #setter: #setAlpha, #start: 50, #end: 0, #ms: 500, #group: "titles"]
  script("node_tween").new(Args)
  buttons.continue.setVisible(1)
end

on setupMap me
  stgs = save_mgr.stages
  savedGame = save_mgr.savedGame
  showStatueAnim = 0
  maxStage = 1
  vis_rest = []
  repeat with i = 1 to 5
    dots = levelDots[i]
    if i > stgs.count then
      call(#setVisible, dots, 0)
      next repeat
    end if
    lvls = stgs[i]
    vis = 0
    repeat with j = 1 to 10
      b = dots[j]
      if voidp(lvls) then
        lvl = VOID
      else
        if lvls.count < j then
          lvl = VOID
        else
          lvl = lvls[j]
        end if
      end if
      unlocked = lvl.unlocked
      if unlocked then
        if (savedGame.stage = i) and (savedGame.level = j) then
          b.setMember(currentDotMem)
        else
          if lvl.starred then
            b.setMember(starMem)
          else
            b.setMember(dotmem)
          end if
        end if
        b.setVisible(1)
        vis = 1
        if j = 2 then
          maxStage = max(maxStage, i)
        end if
        next repeat
      end if
      b.setVisible(0)
    end repeat
    if vis then
      vis_rest.add(i)
    end if
  end repeat
  repeat with i = 2 to 5
    if i = currentLevel.stage then
      if save_mgr.seenItems.getOne(i) = 0 then
        save_mgr.seenItem(i)
        if i < 5 then
          construction[i - 1].setVisible(1)
          const_anim[i - 1].setVisible(0)
          restaurants[i - 1].setVisible(0)
          flashNode([#node: construction[i - 1], #group: "titles", #ms: 200, #loopCount: 4, #callback: [#target: me, #handler: #showConstruction]])
        else
          statueCover.setVisible(0)
          restaurants[4].setVisible(0)
          showStatueAnim = 1
        end if
      else
        buttons.continue.setVisible(1)
        restaurants[i - 1].setVisible(1)
        sparkles[i].setVisible(1)
        sparkles[i].setAnimation("sparkle")
        if i < 5 then
          construction[i - 1].setVisible(0)
        else
          statueCover.setVisible(1)
        end if
      end if
      next repeat
    end if
    if i < 5 then
      construction[i - 1].setVisible(0)
    end if
    if vis_rest.getOne(i) then
      restaurants[i - 1].setVisible(1)
      next repeat
    end if
    restaurants[i - 1].setVisible(0)
  end repeat
  if not showStatueAnim then
    r5vis = vis_rest.getOne(5) <> 0
    statueCover.setVisible(r5vis)
    rest5dots.setVisible(r5vis)
  end if
end

on continue me
  if showStatueAnim then
    mgr.setMode(#transform_comic)
  else
    game_mgr.loadLevel()
    mgr.setMode(#startLevel)
  end if
end

on destroy me
  if initialized then
    call(#destroy, buttons)
    if not voidp(timer) then
      timer.destroy()
      timer = VOID
    end if
    statueAnim.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on fadeUp me
  if showStatueAnim then
    cb = [#target: me, #handler: #startStatueAnim]
  else
    cb = VOID
  end if
  Args = [#node: fadeOut, #setter: #setAlpha, #start: 100, #end: 0, #ms: 500, #group: "titles", #callback: cb]
  script("node_tween").new(Args)
end

on fadeDown me
  call(#setVisible, sparkles, 0)
  restaurants[4].setZOffset(0)
  node.updateLocZ()
  Args = [#node: fadeOut, #setter: #setAlpha, #start: 0, #end: 100, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #continue]]
  script("node_tween").new(Args)
end

on enterMode me, previousMode
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  game_mgr.setPaused(0)
  currentLevel = [#stage: game_mgr.currentStage, #level: game_mgr.currentLevel]
  timer.start()
  call(#setVisible, sparkles, 0)
  buttons.continue.setCallback([#target: me, #handler: #fadeDown])
  me.setupMap(stagedata)
  call(#setVisible, statues, showStatueAnim)
  game_mgr.setPaused(1)
  gameNode.setVisible(0)
  titleNode.setVisible(1)
  node.setVisible(1)
  node.setLoc(point(0, 0))
  call(#setVisible, const_anim, 0)
  fadeOut.setAlpha(100)
  me.fadeUp()
end

on event me, sym
end

on leaveMode me, nextMode
  repeat with s in sparkles
    s.animator.stop()
  end repeat
  node.disembody()
  timer.stop()
  timer.setCallback(VOID)
  call(#setCallback, buttons, VOID)
  node.setLoc(point(-5000, -5000))
  node.setVisible(0)
end
