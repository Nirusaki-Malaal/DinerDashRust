property mgr, name, node_mgr, game_mgr, screen, node, grp, page, hiscore, initialized, arrow

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  grp = node_mgr.getGroup("titles")
  name = #end_comic
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("end_comic"))
  arrow = newButton([#node: node["arrow"], #members: [#rollover: member("help_arrow")], #callback: [#target: me, #handler: #mousePress]])
  node.setVisible(0)
end

on destroy me
  if initialized then
    arrow.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on enterMode me, previousMode, hs
  hiscore = hs
  game_mgr.stopGame()
  getNodeGroup("game").setVisible(0)
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  page = 1
  grp.setVisible(1)
  node.setAnimation("page_" & page)
  node.setVisible(1)
end

on mousePress me
  if page = 3 then
    game_mgr.stopGame()
    if hiscore then
      mgr.setMode(#pre_hiscore, hiscore)
    else
      mgr.setMode(#menu)
    end if
    return 
  end if
  page = page + 1
  node.setAnimation("page_" & page)
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

on event me, sym
end

on leaveMode me, nextMode
  if nextMode.name <> #pre_hiscore then
    node.disembody()
    node.setVisible(0)
  end if
end
