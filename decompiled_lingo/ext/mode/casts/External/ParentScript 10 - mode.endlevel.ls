property mgr, name, node, node_mgr, game_mgr, grp, home, offscreen, bg, win, textBox, served_txt, left_txt, level_txt, levelscore_txt, totalscore_txt, flo_win, flo_lose, text, buttons, button, mousetrap, timer, endStage, endgame, lastLevel, hiscore, dialogY, save_mgr, callList, callCounter, popper, doodle, expertScore, success, menubutton, tween, initialized

on new me, m
  mgr = m
  mm = m.glob.managers
  node_mgr = mm.node
  game_mgr = mm.game
  save_mgr = mm.savegame
  grp = node_mgr.getGroup("titles")
  name = #endlevel
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("endlevel"))
  bg = node["mousetrap"]
  win = node["window"]
  dialogY = win.getLoc()[2]
  repeat with i in ["served_txt", "textbox", "left_txt", "level_txt", "levelscore_txt", "totalscore_txt", "flo_win", "flo_lose"]
    me[symbol(i)] = win[i]
  end repeat
  buttons = [:]
  buttons[#tryagain] = newButton([#node: win["tryagain_button"], #members: [#normal: member("try_again_normal", "gfx.screens"), #rollover: member("try_again_rollover", "gfx.screens")]])
  buttons[#continue] = newButton([#node: win["continue_button"], #members: [#normal: member("continue_normal", "gfx.screens"), #rollover: member("continue_rollover", "gfx.screens")]])
  buttons[#quit] = newButton([#node: win["quit_button"], #members: [#normal: member("quit_normal", "gfx.screens"), #rollover: member("quit_rollover", "gfx.screens")]])
  repeat with i in [#tryagain, #continue, #quit]
    b = buttons[i]
    b.setCallback([#target: me, #handler: #clickButton, #Args: i])
    b.setDisabled(1)
  end repeat
  doodle = win["doodle"]
  mousetrap = newMouseListener([#node: node["mousetrap"]])
  menubutton = game_mgr.buttons.quitgame
  node.setVisible(0)
  home = point(0, 0)
  offscreen = point(-5000, -5000)
  node.setZOffset(10000000)
  node.moveToFront()
  popper = script("popanim", "scripts.mode").new()
  callList = []
  cb = [#target: me, #handler: #callNext]
  repeat with i in [served_txt, left_txt, levelscore_txt, totalscore_txt]
    callList.add([#ms: 250, #callback: [#target: popper, #handler: #pop, #Args: [#node: i, #poploc: point(300, 100), #callback: cb, #timergroup: "titles"]]])
  end repeat
  callList[1].ms = 500
  callList[3].callback.Args[#poploc] = point(-300, -100)
  callList[4].callback.Args[#poploc] = point(-300, -100)
  callCounter = 1
end

on destroy me
  if initialized then
    me.destroyTimer()
    call(#destroy, buttons)
    mousetrap.destroy()
    node.destroy()
    popper.destroy()
  end if
  voidRefs(me)
end

on showDialog me
  Args = [#node: win, #setter: #setY, #start: dialogY - 600, #end: dialogY, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #startDisplay]]
  me.destroyTimer()
  timer = timedCall(500, [#target: me, #handler: #startTween, #Args: Args], "titles")
end

on startDisplay me
  if game_mgr.playMode = #career then
    callCounter = 1
  else
    callCounter = 3
  end if
  me.callNext()
end

on displayDone me
  call(#setDisabled, buttons, 0)
  if success then
    buttons.continue.node.setVisible(1)
  end if
  if expertScore then
    win["expert_star"].setVisible(1)
    script("starpop").new(win["expert_star"])
  end if
end

on callNext me
  if voidp(callList) then
    return 
  end if
  if (callCounter < 1) or (callCounter > callList.count) then
    SndSFX("sfx_food_ready_1_snd")
    me.displayDone()
    return 
  end if
  cb = callList[callCounter]
  me.destroyTimer()
  timer = timedCall(cb.ms, cb.callback, "titles")
  callCounter = callCounter + 1
end

on goMode me, m
  mgr.setMode(m, hiscore)
end

on hideDialog me
  Args = [#node: win, #setter: #setY, #start: dialogY, #end: dialogY - 600, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #fadeDown]]
  me.startTween(Args)
end

on startTween me, Args
  if not voidp(tween) then
    tween.destroy()
  end if
  tween = script("node_tween").new(Args)
end

on fadeDown me
  menubutton.node.setZOffset(0)
  if endStage then
    cb = [#target: mgr, #handler: #setMode, #Args: #startstage]
  else
    if lastLevel then
      cb = [#target: me, #handler: #goMode, #Args: #end_comic]
    else
      cb = [#target: me, #handler: #fadeUp]
    end if
  end if
  Args = [#node: bg, #setter: #setAlpha, #start: 50, #end: 100, #ms: 500, #group: "titles", #callback: cb]
  me.startTween(Args)
end

on fadeUp me
  game_mgr.loadLevel()
  if game_mgr.room.hasNewItems() then
    cb = [#target: me, #handler: #showNewThing]
    Args = [#node: bg, #setter: #setAlpha, #start: 100, #end: 0, #ms: 500, #group: "titles", #callback: cb]
  else
    cb = [#target: mgr, #handler: #setMode, #Args: #startLevel]
    Args = [#node: bg, #setter: #setAlpha, #start: 100, #end: 50, #ms: 500, #group: "titles", #callback: cb]
  end if
  me.destroyTimer()
  timer = timedCall(500, [#target: me, #handler: #startTween, #Args: Args], "titles")
end

on showNewThing me
  menubutton.node.setZOffset(50000000)
  me.destroyTimer()
  game_mgr.room.flashItems()
  timer = timedCall(1800, [#target: me, #handler: #doneShowing], "titles")
end

on doneShowing me
  menubutton.node.setZOffset(0)
  Args = [#node: bg, #setter: #setAlpha, #start: 0, #end: 50, #ms: 500, #group: "titles", #callback: [#target: mgr, #handler: #setMode, #Args: #startLevel]]
  me.startTween(Args)
end

on clickButton me, sym
  if sym = #quit then
    game_mgr.stopGame()
    if hiscore then
      mgr.setMode(#pre_hiscore, hiscore)
    else
      mgr.setMode(#menu)
    end if
  else
    if sym = #tryagain then
      game_mgr.loadLevel()
      mgr.setMode(#startLevel)
    else
      if sym = #continue then
        me.destroyTimer()
        timer = timedCall(100, [#target: me, #handler: #hideDialog], "titles")
      end if
    end if
  end if
end

on destroyTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on enterMode me, previousMode, data
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  menubutton.node.setZOffset(50000000)
  if voidp(data) and (previousMode.name = #quit) then
    return 
  end if
  doodle.setVisible(0)
  m = game_mgr.playMode
  game_mgr.setPaused(1)
  endStage = data[#endStage]
  lastLevel = data[#lastLevel]
  endgame = data[#endgame]
  hiscore = data[#hiscore]
  score = data.score.actual
  requiredScore = data.score[#required]
  expertScore = data[#expertScore]
  call(#setVisible, [levelscore_txt, totalscore_txt, served_txt, left_txt], 0)
  buttons.continue.node.setVisible(0)
  textBox.setText(data[#text])
  levelscore_txt.setText(commafy(string(score)))
  totalscore_txt.setText(commafy(string(game_mgr.totalScore)))
  if m = #career then
    served_txt.setText(string(data.customers.served))
    left_txt.setText(string(data.customers.left))
    level_txt.setText(string(data[#stage]) & "-" & string(data[#level]))
  else
    served_txt.setText(EMPTY)
    left_txt.setText(EMPTY)
  end if
  win["expert_star"].setVisible(0)
  success = data[#success]
  if not success then
    buttons.continue.node.setVisible(0)
    SndSFX("sfx_level_fail")
    if not endgame then
      buttons.tryagain.node.setVisible(1)
      buttons.quit.node.setVisible(0)
    else
      buttons.tryagain.node.setVisible(0)
      buttons.quit.node.setVisible(1)
    end if
    flo_win.setVisible(0)
    flo_lose.setVisible(1)
  else
    SndSFX("sfx_level_complete")
    buttons.tryagain.node.setVisible(0)
    buttons.quit.node.setVisible(0)
    if stringp(data[#doodle]) then
      m = member(data.doodle, "doodles")
      if m.number <> -1 then
        doodle.setMember(m)
        doodle.setVisible(1)
      end if
    end if
    flo_win.setVisible(1)
    flo_lose.setVisible(0)
  end if
  grp.moveToFront()
  node.moveToFront()
  grp.setVisible(1)
  node.setVisible(1)
  node.setLoc(home)
  win.setY(dialogY - 600)
  Args = [#node: bg, #setter: #setAlpha, #start: 0, #end: 50, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #showDialog]]
  me.startTween(Args)
  call(#setDisabled, buttons, 0)
end

on event me, sym
end

on leaveMode me, nextMode
  menubutton.node.setZOffset(0)
  node.disembody()
  call(#setDisabled, buttons, 1)
  grp.setVisible(0)
  node.setVisible(0)
  node.setLoc(offscreen)
  me.destroyTimer()
end
