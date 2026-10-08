property mgr, name, node_mgr, game_mgr, gameNode, titleNode, node, buttons, levelButtons, stagedata, selectedDot, selectedDotMem, dotMems, contMems, starMems, selectedStarMem, selectedContMem, save_mgr, savedGame, selectedLevel, statueCover, rest5dots, restaurants, initialized, cheat
global glob

on new me, m
  mgr = m
  mgrs = m.glob.managers
  node_mgr = mgrs.node
  game_mgr = mgrs.game
  save_mgr = mgrs.savegame
  gameNode = node_mgr.getGroup("game")
  titleNode = node_mgr.getGroup("titles")
  dotMems = [:]
  starMems = [:]
  contMems = [:]
  selectedDotMem = member("leveldot_selected", "gfx.screens")
  selectedStarMem = member("leveldot_star_selected", "gfx.screens")
  selectedContMem = member("lvlcontinue_selected", "gfx.screens")
  d = member("leveldot_normal", "gfx.screens")
  s = member("leveldot_star", "gfx.screens")
  c = member("lvlcontinue_normal", "gfx.screens")
  repeat with i in [#normal, #down, #rollover, #toggle]
    dotMems[i] = d
    starMems[i] = s
    contMems[i] = c
  end repeat
  dotMems[#disabled] = selectedDotMem
  starMems[#disabled] = selectedStarMem
  contMems[#disabled] = selectedContMem
  name = #career
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  cheat = glob.managers.config.overall.cheat
  node = titleNode.newNode(getNodeData("career_setup"))
  node.setVisible(0)
  buttons = [:]
  mems = [#rollover: member("button_back_to_menu_on", "gfx.screens"), #normal: member("button_back_to_menu", "gfx.screens")]
  buttons[#menu] = newButton([#node: node["menu_button"], #members: mems])
  if cheat then
    b = newButton([#node: node["lockall_button"]])
    buttons[#lock] = b
    b = newButton([#node: node["unlockall_button"]])
    buttons[#unlock] = b
  else
    call(#setVisible, [node["lockall_button"], node["unlockall_button"]], 0)
  end if
  mems = [#rollover: member("button_new_game_up", "gfx.screens"), #normal: member("button_new_game", "gfx.screens")]
  buttons[#newGame] = newButton([#node: node["newgame_button"], #members: mems])
  mems = [#rollover: member("button_continue_on", "gfx.screens"), #normal: member("button_continue", "gfx.screens")]
  buttons[#continue] = newButton([#node: node["play_button"], #members: mems])
  restaurants = []
  repeat with i = 2 to 4
    restaurants.add(node["rest_" & i])
    node["const_" & i].destroy()
    node["const_anim_" & i].destroy()
  end repeat
  node["statue_1"].destroy()
  node["statue_2"].destroy()
  node["fadeout"].destroy()
  restaurants.add(node["rest_5"])
  statueCover = node["statue_cover"]
  rest5dots = node["rest5_dots"]
  Args = [#members: dotMems]
  levelButtons = []
  Args[#nosound] = 1
  repeat with i = 1 to 5
    node["sparkle_" & i].destroy()
    lvl = []
    repeat with j = 1 to 10
      Args[#node] = node["lvlbutton_" & i & "_" & j]
      lvl.add(newButton(Args))
    end repeat
    levelButtons.add(lvl)
  end repeat
end

on setupMap me, stgs
  stgs = save_mgr.stages
  savedGame = save_mgr.savedGame
  vis_rest = []
  repeat with i = 1 to 5
    if i > stgs.count then
      call(#setDisabled, levelButtons[i], 1)
      call(#setVisible, levelButtons[i], 0)
      next repeat
    end if
    lvls = stgs[i]
    vis = 0
    repeat with j = 1 to 10
      b = levelButtons[i][j]
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
          b.setMembers(contMems)
        else
          if lvl.starred then
            b.setMembers(starMems)
          else
            b.setMembers(dotMems)
          end if
        end if
        b.setCallback([#target: me, #handler: #clickLevel, #Args: [#stage: i, #level: j]])
        b.setVisible(1)
        if (selectedLevel.stage = i) and (selectedLevel.level = j) then
          selectedDot = b
          b.setDisabled(1)
        else
          b.setDisabled(0)
        end if
        vis = 1
        next repeat
      end if
      b.setCallback(VOID)
      b.setVisible(0)
    end repeat
    if vis then
      vis_rest.add(i)
    end if
  end repeat
  repeat with i = 2 to 5
    if vis_rest.getOne(i) then
      restaurants[i - 1].setVisible(1)
      next repeat
    end if
    restaurants[i - 1].setVisible(0)
  end repeat
  r5vis = vis_rest.getOne(5) <> 0
  statueCover.setVisible(r5vis)
  rest5dots.setVisible(r5vis)
  restaurants[4].setVisible(r5vis)
end

on clickLevel me, Args, btn
  selectedLevel = Args
  if (selectedLevel.level = savedGame.level) and (selectedLevel.stage = savedGame.stage) then
    buttons.newGame.setVisible(0)
    buttons.continue.setVisible(1)
  else
    buttons.newGame.setVisible(1)
    buttons.continue.setVisible(0)
  end if
  if not voidp(selectedDot) then
    selectedDot.setDisabled(0)
  end if
  selectedDot = btn
  btn.setDisabled(1)
end

on continueGame me
  if voidp(savedGame) then
    savedGame = [#stage: 1, #level: 1, #score: 0]
  end if
  game_mgr.newGame(savedGame)
  game_mgr.loadLevel()
  mgr.setMode(#startLevel)
end

on newGame me
  if voidp(selectedLevel) then
    return 
  end if
  sg = selectedLevel.duplicate()
  sg[#score] = 0
  save_mgr.savegame(sg)
  if (selectedLevel.level = 1) and (selectedLevel.stage = 1) then
    mgr.setMode(#comic)
  else
    game_mgr.newGame(selectedLevel)
    game_mgr.loadLevel()
    mgr.setMode(#startLevel)
  end if
end

on destroy me
  if initialized then
    call(#destroy, buttons)
    buttons.deleteAll()
    repeat with ls in levelButtons
      call(#destroy, ls)
      ls.deleteAll()
    end repeat
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
  savedGame = save_mgr.savedGame
  selectedLevel = savedGame.duplicate()
  me.setupMap(stagedata)
  game_mgr.setPlayMode(#career)
  gameNode.setVisible(0)
  titleNode.setVisible(1)
  node.setVisible(1)
  node.setLoc(point(0, 0))
  newGame = (selectedLevel.level = 1) and (selectedLevel.stage = 1)
  buttons.newGame.setCallback([#target: me, #handler: #newGame])
  buttons.newGame.setVisible(newGame)
  buttons.continue.setCallback([#target: me, #handler: #continueGame])
  buttons.continue.setVisible(not newGame)
  buttons.menu.setCallback([#target: mgr, #handler: #setMode, #Args: #menu])
  if cheat then
    buttons.lock.setCallback([#target: me, #handler: #lockAll])
    buttons.unlock.setCallback([#target: me, #handler: #unlockAll])
  end if
end

on lockAll me
  save_mgr.lockAll()
  selectedLevel = [#stage: 1, #level: 1]
  buttons.newGame.setVisible(1)
  buttons.continue.setVisible(0)
  me.setupMap()
end

on unlockAll me
  save_mgr.unlockAll()
  selectedLevel = [#stage: 5, #level: 10]
  buttons.newGame.setVisible(0)
  buttons.continue.setVisible(1)
  me.setupMap()
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  call(#setCallback, buttons, VOID)
  call(#mouseLeave, buttons)
  repeat with ls in levelButtons
    call(#setCallback, ls, VOID)
    call(#mouseLeave, ls)
  end repeat
  node.setLoc(point(-5000, -5000))
  node.setVisible(0)
  node.disembody()
end
