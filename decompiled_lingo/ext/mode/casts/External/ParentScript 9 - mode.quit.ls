property mgr, name, node, node_mgr, game_mgr, event_mgr, grp, home, offscreen, buttons, mousetrap, timer, lastMode, initialized

on new me, m
  mgr = m
  mgrs = m.glob.managers
  node_mgr = mgrs.node
  game_mgr = mgrs.game
  event_mgr = mgrs.event
  name = #quit
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  grp = node_mgr.getGroup("titles")
  node = grp.newNode(getNodeData("quit"))
  buttons = [:]
  buttons[#no] = newButton([#node: node["nobutton"], #members: [#normal: member("Quit_No_off"), #rollover: member("Quit_No_on")]])
  buttons[#yes] = newButton([#node: node["yesbutton"], #members: [#normal: member("Quit_Yes_off"), #rollover: member("Quit_Yes_on")]])
  mousetrap = newMouseListener([#node: node["mousetrap"]])
  node.setVisible(0)
  bg = node["wnd_bg"]
  home = point(0, 0)
  offscreen = point(-5000, -5000)
  node.setZOffset(10000000)
  node.moveToFront()
end

on destroy me
  if initialized then
    me.destroyTimer()
    call(#destroy, buttons)
    mousetrap.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on clickButton me, sym
  if sym = #no then
    me.destroyTimer()
    node.setVisible(0)
    node.setLoc(offscreen)
    updateStage()
    mgr.setMode(lastMode.name, VOID, 1)
  else
    if lastMode.name = #menu then
      halt()
    else
      if lastMode.name = #play then
        game_mgr.stopGame()
      end if
    end if
    lastMode.leaveMode(mgr.modes.menu)
    mgr.setMode(#menu)
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
  lastMode = previousMode
  game_mgr.setPaused(1)
  event_mgr.setGroupPaused(1, "titles")
  buttons.yes.setCallback([#target: me, #handler: #clickButton, #Args: #yes])
  buttons.no.setCallback([#target: me, #handler: #clickButton, #Args: #no])
  grp.moveToFront()
  node.moveToFront()
  grp.setVisible(1)
  node.setVisible(1)
  node.setLoc(home)
end

on event me, sym
end

on leaveMode me, nextMode
  event_mgr.setGroupPaused(0, "titles")
  node.disembody()
  game_mgr.setPaused(0)
  call(#setCallback, buttons, VOID)
  node.setVisible(0)
  node.setLoc(offscreen)
  me.destroyTimer()
end
