property mgr, name, node_mgr, game_mgr, gameNode, titleNode, node, buttons, stageButtons, hilite, save_mgr, highestStage, diffDialog, diffButtons, mousetrap, stagedata, currentStage, leveldata, statueCover, sparkles, initialized
global glob

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  save_mgr = m.glob.managers.savegame
  gameNode = node_mgr.getGroup("game")
  titleNode = node_mgr.getGroup("titles")
  name = #endless
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = titleNode.newNode(getNodeData("endless_setup"))
  node.setVisible(0)
  buttons = [:]
  statueCover = node["statue_cover"]
  buttons[#menu] = newButton([#node: node["menu_button"]])
  diffDialog = node["diff_dialog"]
  diffButtons = []
  repeat with i = 1 to 3
    z = "diff_" & i
    diffButtons.add(newButton([#node: diffDialog[z], #members: [#normal: member(z & "_off"), #rollover: member(z & "_on")]]))
  end repeat
  mousetrap = newMouseListener([#node: diffDialog["mousetrap"]])
  buttons[#closeDiff] = newButton([#node: diffDialog["closediff"]])
  stagedata = glob.managers.config.game.endless.stages
  currentStage = 1
  stageButtons = []
  repeat with i = 1 to 5
    stageButtons.add(script("endless_stagebutton", "scripts.mode").new([#node: node["rest_" & i], #sparkle: node["sparkle_" & i]]))
  end repeat
end

on destroy me
  if initialized then
    call(#destroy, buttons)
    mousetrap.destroy()
    call(#destroy, stageButtons)
    call(#destroy, diffButtons)
    node.destroy()
  end if
  voidRefs(me)
end

on enterMode me, previousMode
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  diffDialog.setVisible(0)
  st = save_mgr.stages
  highestStage = 1
  repeat with i = 1 to st.count
    lvl = st[i]
    unlck = 0
    repeat with j in lvl
      if j.unlocked then
        unlck = 1
        exit repeat
      end if
    end repeat
    if not unlck then
      exit repeat
    end if
    highestStage = i
  end repeat
  gameNode.setVisible(0)
  titleNode.setVisible(1)
  node.setVisible(1)
  node.setLoc(point(0, 0))
  game_mgr.setPlayMode(#endless)
  buttons.menu.setCallback([#target: mgr, #handler: #setMode, #Args: #menu])
  me.setupStages()
  me.setupLevels()
end

on setupStages me
  repeat with i = 1 to 5
    sb = stageButtons[i]
    if i > highestStage then
      sb.setCallback(VOID)
      sb.setVisible(0)
      next repeat
    end if
    sb.setCallback([#target: me, #handler: #clickStage, #Args: i])
    sb.setVisible(1)
  end repeat
  statueCover.setVisible(highestStage = 5)
end

on setupLevels me
  repeat with i = 1 to 3
    diffButtons[i].setCallback([#target: me, #handler: #clickLevel, #Args: i])
  end repeat
  buttons[#closeDiff].setCallback([#target: me, #handler: #closeDiff])
end

on closeDiff me
  diffDialog.setVisible(0)
end

on clickStage me, num
  currentStage = num
  diffDialog.setVisible(1)
end

on clickLevel me, lvl
  game_mgr.newGame([#stage: currentStage, #difficulty: lvl, #mode: #endless])
  game_mgr.loadLevel()
  mgr.setMode(#startLevel)
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  call(#setCallback, stageButtons, VOID)
  call(#mouseLeave, stageButtons)
  call(#setCallback, diffButtons, VOID)
  call(#mouseLeave, diffButtons)
  node.setLoc(point(-5000, -5000))
  node.setVisible(0)
end
