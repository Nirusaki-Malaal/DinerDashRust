property mgr, name, node_mgr, save_mgr, node, buttons, username_txt, initialized, hairtimer, eyetimer, hair, eyes

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  save_mgr = m.glob.managers.savegame
  eyetimer = script("timeout").new()
  eyetimer.setGroup("titles")
  eyetimer.setCallback([#target: me, #handler: #doEyeAnim])
  hairtimer = script("timeout").new()
  hairtimer.setGroup("titles")
  hairtimer.setCallback([#target: me, #handler: #doHairAnim])
  name = #menu
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  grp = node_mgr.getGroup("titles")
  node = grp.newNode(getNodeData("title_screen"))
  node.setZOffset(500000)
  hair = node["hair"]
  eyes = node["eyes"]
  buttons = [:]
  snds = [#rollover: "sfx_menu_rollover", #press: "sfx_menu_select"]
  repeat with i in [#career, #endless, #help, #hiscore, #options, #quit, #login]
    mems = [#normal: member("button_" & i, "gfx.screens"), #rollover: member("button_" & i & "_on", "gfx.screens")]
    bnode = node[string(i) & "_button"]
    buttons[i] = newButton([#node: bnode, #members: mems, #sounds: snds])
  end repeat
  username_txt = node["username_txt"]
end

on destroy me
  if initialized then
    call(#destroy, buttons)
    node.destroy()
    hairtimer.destroy()
    eyetimer.destroy()
  end if
  voidRefs(me)
end

on goMode me, sym
  if ((sym = #career) or (sym = #endless)) and voidp(save_mgr.currentProfile) then
    mgr.setMode(#login, sym)
    return 
  end if
  mgr.setMode(sym)
end

on doHairAnim me
  hairtimer.stop()
  hair.setAnimation("wave", [#loopCount: 1 + random(2), #frameMS: 200])
  hairtimer.setPeriod(1000 + random(1000))
  hairtimer.start()
end

on doEyeAnim me
  eyetimer.stop()
  eyes.setAnimation("blink", [#loopCount: random(2), #frameMS: 40])
  eyetimer.setPeriod(1500 + random(2000))
  eyetimer.start()
end

on enterMode me, previousMode
  if not initialized then
    me.init()
  end if
  uname = save_mgr.getCurrentProfile()
  if not voidp(uname) then
    username_txt.setText(uname)
    username_txt.setVisible(1)
    node["user_bg"].setVisible(1)
    buttons.login.setVisible(1)
  else
    node["user_bg"].setVisible(0)
    username_txt.setVisible(0)
    buttons.login.setVisible(0)
  end if
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  repeat with i = 1 to buttons.count
    p = buttons.getPropAt(i)
    buttons[i].setCallback([#target: me, #handler: #goMode, #Args: p])
  end repeat
  SndMusicStart("music_intro")
  node_mgr.getGroup("game").setVisible(0)
  node_mgr.getGroup("titles").setVisible(1)
  node.setVisible(1)
  node.setLoc(point(0, 0))
  hairtimer.setPeriod(500 + random(1000))
  hairtimer.start()
  eyetimer.setPeriod(500 + random(1000))
  eyetimer.start()
end

on event me, sym
end

on leaveMode me, nextMode
  eyetimer.stop()
  hairtimer.stop()
  hair.animator.stop()
  eyes.animator.stop()
  call(#setCallback, buttons, VOID)
  call(#mouseLeave, buttons)
  n = nextMode.name
  if (n <> #options) and (n <> #quit) then
    node.setVisible(0)
    node.setLoc(point(-5000, -5000))
    node.disembody()
  end if
end
