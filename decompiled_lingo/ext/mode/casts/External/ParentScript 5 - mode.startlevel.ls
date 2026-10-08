property mgr, name, node, win, node_mgr, grp, home, offscreen, textBox, dialogY, game_mgr, button, mousetrap, timer, bg, otherstuff, gameGrp, initialized, menubutton, tween, stageSounds

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  gameGrp = node_mgr.getGroup("game")
  stageSounds = ["sfx_r_diner", "sfx_r_tiki", "sfx_r_seafood", "sfx_r_chez", "sfx_r_goddess"]
  name = #startLevel
  me.init()
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  grp = node_mgr.getGroup("titles")
  node = grp.newNode(getNodeData("startlevel"))
  win = node["window"]
  dialogY = win.getLoc()[2]
  bg = node["mousetrap"]
  mems = [#normal: member("lets_play_normal", "gfx.screens"), #rollover: member("lets_play_rollover", "gfx.screens")]
  button = newButton([#node: win["okbutton"], #members: mems])
  mousetrap = newMouseListener([#node: node["mousetrap"]])
  node.setVisible(0)
  menubutton = game_mgr.buttons.quitgame
  home = point(0, 0)
  offscreen = point(-5000, -5000)
  node.setZOffset(999999)
  node.moveToFront()
end

on destroy me
  if initialized then
    me.destroyTimer()
    button.destroy()
    mousetrap.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on destroyTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on enterMode me, previousMode, data
  gameGrp.setVisible(1)
  menubutton.node.setZOffset(50000000)
  if game_mgr.playMode = #career then
    m = member(member("startlvl_text_" & game_mgr.currentStage & "-" & game_mgr.currentLevel, "leveltext"))
    if m.number <> -1 then
      win["text"].setMember(m)
    end if
    str = "level_" & game_mgr.currentStage & "_" & game_mgr.currentLevel
    win["pic"].setAnimation(str, [#loopCount: 1])
    win["pic"].setVisible(1)
  else
    win["text"].setMember(member("startlvl_text_endless", "leveltext"))
    win["pic"].setVisible(0)
  end if
  grp.setVisible(1)
  bg.setVisible(1)
  win.setVisible(1)
  win.setY(dialogY - 600)
  bg.setAlpha(50)
  node.setVisible(1)
  node.setLoc(home)
  n = previousMode.name
  if n <> #endlevel then
    me.showDialog()
  else
    me.showDialog()
  end if
end

on showDialog me
  st = game_mgr.currentStage
  if st < stageSounds.count then
    SndSFX(stageSounds[st])
  end if
  Args = [#node: win, #setter: #setY, #start: dialogY - 600, #end: dialogY, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #activateDialog]]
  me.destroyTimer()
  timer = timedCall(500, [#target: me, #handler: #startTween, #Args: Args], "titles")
end

on activateDialog me
  button.setCallback([#target: me, #handler: #clickOK])
end

on clickOK me
  me.hideDialog()
end

on hideDialog me
  Args = [#node: win, #setter: #setY, #start: dialogY, #end: dialogY - 600, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #fadeUp]]
  me.startTween(Args)
end

on fadeUp me
  Args = [#node: bg, #setter: #setAlpha, #start: 50, #end: 0, #ms: 500, #group: "titles", #callback: [#target: me, #handler: #goPlay]]
  me.startTween(Args)
end

on startTween me, Args
  if not voidp(tween) then
    tween.destroy()
  end if
  tween = script("node_tween").new(Args)
end

on goPlay me
  me.destroyTimer()
  node.setVisible(0)
  node.setLoc(offscreen)
  mgr.setMode(#play)
end

on event me, sym
end

on leaveMode me, nextMode
  if not voidp(tween) then
    tween.destroy()
  end if
  menubutton.node.setZOffset(0)
  button.setCallback(VOID)
  node.setVisible(0)
  node.setLoc(offscreen)
  me.destroyTimer()
end
